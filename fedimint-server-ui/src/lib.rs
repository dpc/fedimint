use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Form, State};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use fedimint_core::module::ApiAuth;
use fedimint_server_ui_common::DynUiBackend;
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Debug, Deserialize)]
struct SetupInput {
    password: String,
    name: String,
    federation_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginInput {
    password: String,
}

#[derive(Debug, Deserialize)]
struct PeerInfoInput {
    peer_info: String,
}

// Common base HTML layout for all pages
fn base_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" integrity="sha384-T3c6CoIi6uLrA9TneNEoa7RxnatzjcDSCmG1MXxSR1GAsXEV/Dwwykc2MPK8M2HN" crossorigin="anonymous";
                style {
                    r#"
                    body {
                        background-color: #f8f9fa;
                        padding-top: 2rem;
                        padding-bottom: 2rem;
                    }
                    
                    .header-title {
                        color: #0d6efd;
                        margin-bottom: 2rem;
                    }
                    
                    .card {
                        border: none;
                        box-shadow: 0 0.5rem 1rem rgba(0, 0, 0, 0.15);
                        border-radius: 0.5rem;
                        margin-bottom: 2rem;
                    }
                    
                    .card-header {
                        background-color: #fff;
                        border-bottom: 1px solid rgba(0, 0, 0, 0.125);
                        padding: 1.5rem;
                    }
                    
                    .card-body {
                        padding: 2rem;
                    }
                    
                    .form-label {
                        font-weight: 500;
                    }
                    
                    .field-description {
                        color: #6c757d;
                        font-size: 0.875rem;
                        margin-top: 0.25rem;
                    }
                    
                    .form-control {
                        max-width: 100%;
                    }
                    
                    .form-group {
                        margin: 0 auto;
                        max-width: 400px;
                    }
                    
                    .btn {
                        min-width: 200px; /* Make all buttons wider */
                        padding: 0.6rem 2rem;
                    }
                    
                    .button-container {
                        text-align: center;
                        margin-top: 2rem;
                    }
                    
                    /* For the dashboard buttons that appear side by side */
                    .protected-area .button-container .btn {
                        min-width: 160px;
                        margin-bottom: 0.5rem;
                    }
                    
                    .error-message {
                        color: #dc3545;
                        margin-top: 1rem;
                        font-weight: 500;
                    }
                    
                    .alert-info {
                        background-color: #e8f4f8;
                        border-color: #bee5eb;
                    }
                    
                    .connection-code {
                        background-color: #f8f9fa;
                        border: 1px solid #dee2e6;
                        border-radius: 0.25rem;
                        padding: 1rem;
                        overflow-x: auto;
                        font-family: monospace;
                        margin-bottom: 1rem;
                        word-break: break-all;
                        color: #000; /* Set text color explicitly to black */
                    }
                    
                    /* Explicitly set code element color */
                    .connection-code code {
                        color: #000 !important; /* Force black color with !important */
                    }
                    
                    /* Consistent button width for the setup UI */
                    .setup-btn {
                        width: 75%;
                        max-width: 300px;
                        margin: 0 auto;
                    }
                    
                    @media (min-width: 992px) {
                        .narrow-container {
                            max-width: 500px;
                        }
                    }
                    "#
                }
            }
            body {
                div class="container" {
                    div class="row justify-content-center" {
                        div class="col-md-8 col-lg-5 narrow-container" {
                            header class="text-center" {
                                h1 class="header-title" { "Fedimint Guardian UI" }
                            }

                            div class="card" {
                                div class="card-body" {
                                    (content)
                                }
                            }
                        }
                    }
                }
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js" integrity="sha384-C6RzsynM9kWDrMNeT87bh95OGNyZPhcTNXj1NW7RuBCsyN/o0jlpcV8Qyq46cDfL" crossorigin="anonymous" {}
            }
        }
    }
}

// GET handler for the /setup route (display the setup form)
async fn setup_form(State(state): UiState) -> impl IntoResponse {
    // Check if we already have local params set
    // if config_api.local_params().await.is_some() {
    if state.backend.are_local_params_set().await {
        return Redirect::to("/federation-setup").into_response();
    }

    // Password not set, render the setup form
    let content = html! {
        h2 class="mb-4 text-center" { "Set Guardian Parameters" }
        form method="post" action="/" {
            div class="form-group mb-4" {
                label for="name" class="form-label" { "Guardian Name" }
                input type="text" class="form-control" id="name" name="name" placeholder="Your guardian name" required;
            }

            div class="form-group mb-4" {
                label for="federation_name" class="form-label" { "Federation Name (optional)" }
                input type="text" class="form-control" id="federation_name" name="federation_name" placeholder="Federation name";
                div class="field-description" {
                    "The federation name needs to be set by exactly one guardian."
                }
            }

            div class="form-group mb-4" {
                label for="password" class="form-label" { "Guardian Password" }
                input type="password" class="form-control" id="password" name="password" placeholder="Secure password" required;
            }

            div class="button-container" {
                button type="submit" class="btn btn-primary setup-btn" { "Set Parameters" }
            }
        }
    };

    Html(base_layout("Setup Fedimint Guardian", content).into_string()).into_response()
}

// POST handler for the /setup route (process the password setup form)
async fn setup_submit(State(state): UiState, Form(input): Form<SetupInput>) -> impl IntoResponse {
    // Create ApiAuth from password
    let auth = ApiAuth(input.password.clone());

    // Set local parameters using the config gen API, pass name and federation_name
    // directly
    match config_api
        .set_local_parameters(auth, input.name, input.federation_name)
        .await
    {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(e) => {
            // Show error on setup page
            let content = html! {
                h2 class="mb-4 text-center" { "Setup Failed" }
                div class="alert alert-danger" { (e.to_string()) }
                div class="button-container" {
                    a href="/" class="btn btn-primary setup-btn" { "Try Again" }
                }
            };

            Html(base_layout("Setup Error", content).into_string()).into_response()
        }
    }
}

// GET handler for the /login route (display the login form)
async fn login_form(State(state): UiState) -> impl IntoResponse {
    // Check if local params are set
    if config_api.local_params().await.is_none() {
        return Redirect::to("/").into_response();
    }

    let content = html! {
        h2 class="mb-4 text-center" { "Guardian Login" }
        form method="post" action="/login" {
            div class="form-group mb-4" {
                label for="password" class="form-label" { "Enter your guardian password" }
                input type="password" class="form-control" id="password" name="password" placeholder="Your password" required;
            }
            div class="button-container" {
                button type="submit" class="btn btn-primary setup-btn" { "Log In" }
            }
        }
    };

    Html(base_layout("Fedimint Guardian Login", content).into_string()).into_response()
}

// POST handler for the /login route (authenticate and set session cookie)
async fn login_submit(
    State(state): UiState,
    jar: CookieJar,
    Form(input): Form<LoginInput>,
) -> impl IntoResponse {
    let auth = match config_api.auth().await {
        Some(auth) => auth,
        None => return Redirect::to("/").into_response(),
    };

    // Check if password matches
    if auth.0 == input.password {
        // Password matches, create a session cookie
        let mut cookie = Cookie::new("session", input.password);
        cookie.set_http_only(true);
        cookie.set_same_site(Some(SameSite::Lax));

        return (jar.add(cookie), Redirect::to("/federation-setup")).into_response();
    }

    // If we reach here, authentication failed (wrong password)
    let content = html! {
        h2 class="mb-4 text-center" { "Guardian Login" }
        div class="alert alert-danger" role="alert" {
            "Invalid password. Please try again."
        }
        form method="post" action="/login" {
            div class="form-group mb-4" {
                label for="password" class="form-label" { "Enter your guardian password" }
                input type="password" class="form-control" id="password" name="password" placeholder="Your password" required;
            }
            div class="button-container" {
                button type="submit" class="btn btn-primary setup-btn" { "Log In" }
            }
        }
    };

    Html(base_layout("Login Failed", content).into_string()).into_response()
}

// Helper function to check authentication - returns a redirect if
// authentication fails
async fn check_auth(State(state): UiState, jar: &CookieJar) -> Option<Redirect> {
    // Step 1: Get the session cookie if it exists
    let session_password = match jar.get("session") {
        Some(cookie) => cookie.value().to_string(),
        None => return Some(Redirect::to("/login")), // No cookie found, redirect to login
    };

    // Use match to handle the Option and avoid unwrap
    let auth = match config_api.auth().await {
        Some(auth) => auth,
        None => return Some(Redirect::to("/")),
    };

    // Step 3: Check if password matches - direct comparison
    if auth.0 != session_password {
        return Some(Redirect::to("/login"));
    }

    None
}

// GET handler for the /federation-setup route (main federation management page)
async fn federation_setup(State(state): UiState, jar: CookieJar) -> impl IntoResponse {
    // Authenticate with session cookie
    if let Some(redirect) = check_auth(&config_api, &jar).await {
        return redirect.into_response();
    }

    // Create our connection info
    let our_connection_info = config_api
        .local_params()
        .await
        .unwrap()
        .connection_info()
        .encode_base32();

    // Get the list of peers directly from state
    let connected_peers = config_api.connected_peers().await;

    // Render the federation setup page
    let content = html! {
        h2 class="text-center mb-4" { "Federation Setup" }

        // Your connection info section
        section class="mb-4" {
            div class="alert alert-info mb-3" {
                "Share this code with other guardians:"
            }

            div class="connection-code card p-3 mb-3" {
                code { (our_connection_info) }
            }

            div class="text-center" {
                button type="button" class="btn btn-outline-primary setup-btn"
                    onclick=(format!("navigator.clipboard.writeText('{}')", our_connection_info)) {
                    "Copy to Clipboard"
                }
            }
        }

        // Divider
        hr class="my-4" {}

        // Add other guardians section
        section class="mb-4" {
            h4 class="mb-3" { "Connect with Other Guardians" }

            // Connected guardians list (if any)
            @if !connected_peers.is_empty() {
                div class="mb-4" {
                    ul class="list-group mb-4" {
                        @for peer in connected_peers {
                            li class="list-group-item" { (peer) }
                        }
                    }
                }
            }

            // Add guardian form
            form method="post" action="/add-peer" {
                div class="mb-3" {
                    input type="text" class="form-control mb-2" id="peer_info" name="peer_info"
                        placeholder="Paste connection info from another guardian" required;
                }

                div class="text-center" {
                    button type="submit" class="btn btn-primary setup-btn" { "Add Guardian" }
                }
            }
        }

        // Divider
        hr class="my-4" {}

        // Launch section
        section class="mb-4" {
            h4 class="mb-3" { "Launch Federation" }

            // Warning message about point of no return
            div class="alert alert-warning mb-4" {
                "Make sure all information is correct and every guardian is ready before launching the federation. This process cannot be reversed once started."
            }

            div class="text-center" {
                form method="post" action="/start-dkg" {
                    button type="submit" class="btn btn-warning setup-btn" {
                        "🚀 Launch Federation"
                    }
                }
            }
        }
    };

    Html(base_layout("Federation Setup", content).into_string()).into_response()
}

// POST handler for adding peer connection info
async fn add_peer_handler(
    State(state): UiState,
    jar: CookieJar,
    Form(input): Form<PeerInfoInput>,
) -> impl IntoResponse {
    // Authenticate with session cookie
    if let Some(redirect) = check_auth(&config_api, &jar).await {
        return redirect.into_response();
    }

    // Decode peer connection info
    match PeerConnectionInfo::decode_base32(&input.peer_info) {
        Ok(info) => {
            // Use ConfigGenApi to add the peer
            match config_api.add_peer_connection_info(info).await {
                Ok(()) => Redirect::to("/federation-setup").into_response(),
                Err(e) => {
                    // Show error with federation setup
                    let content = html! {
                        h2 class="mb-4 text-center" { "Error Adding Guardian" }
                        div class="alert alert-danger" { (e.to_string()) }
                        div class="button-container" {
                            a href="/federation-setup" class="btn btn-primary setup-btn" { "Back to Setup" }
                        }
                    };

                    Html(base_layout("Error", content).into_string()).into_response()
                }
            }
        }
        Err(e) => {
            // Invalid connection info
            let content = html! {
                h2 class="mb-4 text-center" { "Invalid Connection Info" }
                div class="alert alert-danger" { "The provided connection info is not valid: " (e.to_string()) }
                div class="button-container" {
                    a href="/federation-setup" class="btn btn-primary setup-btn" { "Back to Setup" }
                }
            };

            Html(base_layout("Error", content).into_string()).into_response()
        }
    }
}

// POST handler for starting the DKG process
async fn start_dkg_handler(State(state): UiState, jar: CookieJar) -> impl IntoResponse {
    // Authenticate with session cookie
    if let Some(redirect) = check_auth(&config_api, &jar).await {
        return redirect.into_response();
    }

    // Start DKG using the ConfigGenApi
    match config_api.start_dkg().await {
        Ok(()) => {
            // Show simple DKG success page
            let content = html! {
                h2 class="text-center" { "Federation Initialization Started" }
                div class="alert alert-success my-4" {
                    "The distributed key generation has been started successfully."
                }
                p class="text-center" {
                    "The federation is now being initialized. You can monitor the progress in your server logs."
                }
                p class="text-center mt-3" {
                    "This interface will be available until the DKG process completes."
                }
            };

            Html(base_layout("DKG Started", content).into_string()).into_response()
        }
        Err(e) => {
            // Error starting DKG
            let content = html! {
                h2 class="mb-4 text-center" { "Error Starting Federation" }
                div class="alert alert-danger" { (e.to_string()) }
                div class="button-container" {
                    a href="/federation-setup" class="btn btn-primary setup-btn" { "Back to Setup" }
                }
            };

            Html(base_layout("Error", content).into_string()).into_response()
        }
    }
}

struct UiStateInner {
    backend: DynUiBackend,
}
type SharedUiState = Arc<UiStateInner>;
type UiState = State<SharedUiState>;

// Main function to start the web UI with ConfigGenApi
pub async fn start_web_ui(backend: DynUiBackend, ui_bind: SocketAddr) {
    // Build router with ConfigGenApi as state
    let app = Router::new()
        .route("/", get(setup_form).post(setup_submit))
        .route("/login", get(login_form).post(login_submit))
        .route("/federation-setup", get(federation_setup))
        .route("/add-peer", post(add_peer_handler))
        .route("/start-dkg", post(start_dkg_handler))
        .with_state(Arc::new(UiStateInner { backend }));

    // Run the Axum server
    println!("Federation setup UI running at http://{ui_bind} 🚀");

    let listener = TcpListener::bind(ui_bind)
        .await
        .expect("Failed to bind to port");

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Failed to start server");
}

#[ignore]
#[tokio::test]
async fn start_web_ui_test() {
    use std::collections::BTreeMap;

    use fedimint_core::config::ServerModuleConfigGenParamsRegistry;
    use fedimint_core::db::mem_impl::MemDatabase;
    use fedimint_core::util::SafeUrl;
    use fedimint_server_core::ServerModuleInitRegistry;
    use tokio::sync::mpsc;

    // Create an in-memory database for testing
    let db = MemDatabase::new().into();

    // Create a channel for ConfigGenParams
    let (sender, mut receiver) = mpsc::channel(10);

    // Create dummy settings with correct p2p_url format
    let settings = ConfigGenSettings {
        api_bind: ([127, 0, 0, 1], 8173).into(),
        p2p_bind: ([127, 0, 0, 1], 8174).into(),
        ui_bind: ([127, 0, 0, 1], 8175).into(),
        api_url: SafeUrl::parse(&format!("https://localhost:{}", 8173)).unwrap(),
        p2p_url: SafeUrl::parse(&format!("fedimint://localhost:{}", 8174)).unwrap(),
        networking: NetworkingStack::Tcp,
        modules: ServerModuleConfigGenParamsRegistry::default(),
        meta: BTreeMap::default(),
        registry: ServerModuleInitRegistry::default(),
    };

    // Create a dummy ConfigGenApi
    let config_gen_api = ConfigGenApi::new(settings, db, sender);

    // Spawn the web UI in a separate task
    fedimint_core::task::spawn(
        "web-ui",
        start_web_ui(config_gen_api, ([127, 0, 0, 1], 8175).into()),
    );

    // In the main test thread, wait for a DKG start signal
    if let Some(params) = receiver.recv().await {
        println!(
            "DKG process started with {} peers. Terminating test server...",
            params.peers.len()
        );
    }
}
