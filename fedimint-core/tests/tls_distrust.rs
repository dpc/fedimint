//! Regression coverage for the verifier security floor, independent of root
//! removal.

#![cfg(not(target_family = "wasm"))]

use std::time::Duration;

use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, UnixTime};

#[test]
fn rejects_compromised_intermediate_even_with_trusted_cross_signs() {
    // Deliberately retain both affected roots: root-store cleanup alone must not
    // be the reason these chains fail, including with operator-provided roots.
    let roots = certificates(include_bytes!("fixtures/emudhra/roots.pem"));
    let anchors = roots
        .iter()
        .map(|cert| webpki::anchor_from_trusted_cert(cert).unwrap())
        .collect::<Vec<_>>();
    // Fixed within both leaf certificates' validity periods.
    let time = UnixTime::since_unix_epoch(Duration::from_secs(1_792_497_600));

    for chain in [
        include_bytes!("fixtures/emudhra/direct.pem").as_slice(),
        include_bytes!("fixtures/emudhra/cross.pem").as_slice(),
    ] {
        let chain = certificates(chain);
        let leaf = webpki::EndEntityCert::try_from(&chain[0]).unwrap();
        let result = leaf.verify_for_usage(
            webpki::ALL_VERIFICATION_ALGS,
            &anchors,
            &chain[1..],
            time,
            webpki::KeyUsage::server_auth(),
            None,
            None,
        );
        assert!(matches!(result, Err(webpki::Error::CertDistrusted)));
    }
}

fn certificates(pem: &[u8]) -> Vec<CertificateDer<'static>> {
    CertificateDer::pem_slice_iter(pem)
        .collect::<Result<_, _>>()
        .unwrap()
}
