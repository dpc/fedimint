# Public eMudhra distrust fixtures

These fixtures contain public certificates, not private keys. We captured them
on October 20, 2026 from the disclosure's two TLS test endpoints using
`openssl s_client -showcerts`, without sending HTTP requests or credentials:

- `direct.pem`: `dv-g3b-emudhra.we-have-a-problem.com:443`, leaf first, then
  EM DV TLS CA - G3B and its emSign Root TLS CA - G3 issuer.
- `cross.pem`: `dv-g3b-aaa-emudhra.we-have-a-problem.com:443`, leaf first, then
  the same intermediate key cross-signed by AAA Certificate Services.
- `roots.pem`: the emSign ECC Root CA - G3 and AAA Certificate Services
  certificates from <https://emudhra.we-have-a-problem.com/roots.json>.

The test trusts both roots and uses October 20, 2026 at 12:00 UTC,
inside both leaf validity periods. This isolates the verifier's SPKI distrust
from root removal, certificate expiration, DNS, and network availability.
The verifier must reject both chains even when an operator retains those roots.

The disclosure and the upstream fix are public:
<https://emudhra.we-have-a-problem.com/> and
<https://github.com/rustls/webpki/pull/546>.
