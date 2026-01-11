use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{get, post, put, delete, patch},
    Router,
    middleware,
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use std::collections::HashMap;

// --- Veri Yapıları ---

#[derive(Clone)]
struct AppState {
    sessions: Arc<HashMap<String, String>>,
}

#[derive(Serialize)]
struct TestResponse {
    message: String,
    timestamp: i64,
}

#[derive(Deserialize)]
struct VulnParams {
    file: Option<String>,
    url: Option<String>,
    name: Option<String>,
    id: Option<String>,
    q: Option<String>,
    to: Option<String>,
    cmd: Option<String>,
    xml: Option<String>,
    path: Option<String>,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    email: Option<String>,
    redirect: Option<String>,
    template: Option<String>,
    lang: Option<String>,
    callback: Option<String>,
    debug: Option<String>,
    search: Option<String>,
    user: Option<String>,
    host: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let state = AppState {
        sessions: Arc::new(HashMap::new()),
    };

    let app = Router::new()
        // --- 1. Performance Benchmark ---
        .route("/", get(benchmark_handler))
        .route("/test", get(benchmark_handler))
        .route("/health", get(health_handler))
        .route("/ping", get(ping_handler))

        // --- 2. Sensitive File Exposure ---
        .route("/.env", get(env_handler))
        .route("/.env.backup", get(env_backup_handler))
        .route("/.env.local", get(env_local_handler))
        .route("/.env.production", get(env_prod_handler))
        .route("/config.json", get(config_handler))
        .route("/config.yml", get(config_yml_handler))
        .route("/appsettings.json", get(appsettings_handler))
        .route("/.git/config", get(git_config_handler))
        .route("/.git/HEAD", get(git_head_handler))
        .route("/.git/index", get(git_index_handler))
        .route("/.gitignore", get(gitignore_handler))
        .route("/backup.sql", get(backup_handler))
        .route("/database.sql", get(database_sql_handler))
        .route("/dump.sql", get(dump_sql_handler))
        .route("/phpinfo.php", get(phpinfo_handler))
        .route("/info.php", get(phpinfo_handler))
        .route("/server-status", get(server_status_handler))
        .route("/robots.txt", get(robots_handler))
        .route("/.htaccess", get(htaccess_handler))
        .route("/.htpasswd", get(htpasswd_handler))
        .route("/web.config", get(web_config_handler))
        .route("/.DS_Store", get(ds_store_handler))
        .route("/package.json", get(package_json_handler))
        .route("/composer.json", get(composer_json_handler))
        .route("/Gemfile", get(gemfile_handler))
        .route("/requirements.txt", get(requirements_handler))
        .route("/yarn.lock", get(yarn_lock_handler))
        .route("/.npmrc", get(npmrc_handler))
        .route("/credentials.json", get(credentials_handler))
        .route("/id_rsa", get(ssh_key_handler))
        .route("/.ssh/id_rsa", get(ssh_key_handler))
        .route("/id_rsa.pub", get(ssh_pub_handler))
        .route("/access.log", get(access_log_handler))
        .route("/error.log", get(error_log_handler))
        .route("/application.log", get(app_log_handler))
        .route("/console.log", get(console_log_handler))

        // --- 3. Injection Vulnerabilities ---
        .route("/vuln/lfi", get(lfi_handler))
        .route("/vuln/rfi", get(rfi_handler))
        .route("/vuln/ssrf", get(ssrf_handler))
        .route("/vuln/ssti", get(ssti_handler))
        .route("/vuln/xss", get(xss_handler))
        .route("/vuln/dom-xss", get(dom_xss_handler))
        .route("/vuln/sqli", get(sqli_handler))
        .route("/vuln/sqli-blind", get(sqli_blind_handler))
        .route("/vuln/sqli-time", get(sqli_time_handler))
        .route("/vuln/nosqli", get(nosqli_handler))
        .route("/vuln/rce", get(rce_handler))
        .route("/vuln/xxe", post(xxe_handler))
        .route("/vuln/xpath", get(xpath_injection_handler))
        .route("/vuln/ldap", get(ldap_injection_handler))
        .route("/vuln/path-traversal", get(path_traversal_handler))
        .route("/vuln/cmd-injection", get(cmd_injection_handler))
        .route("/vuln/code-injection", get(code_injection_handler))
        .route("/vuln/template-injection", get(template_injection_handler))
        .route("/vuln/crlf", get(crlf_injection_handler))
        .route("/vuln/header-injection", get(header_injection_handler))

        // --- 4. Authentication & Authorization ---
        .route("/login", post(login_handler))
        .route("/admin/login", post(admin_login_handler))
        .route("/api/login", post(api_login_handler))
        .route("/vuln/auth-bypass", get(auth_bypass_handler))
        .route("/vuln/default-creds", post(default_creds_handler))
        .route("/vuln/weak-password", post(weak_password_handler))
        .route("/vuln/jwt-none", get(jwt_none_handler))
        .route("/vuln/jwt-weak", get(jwt_weak_handler))
        .route("/vuln/session-fixation", get(session_fixation_handler))
        .route("/vuln/password-reset", post(password_reset_handler))
        .route("/api/admin/users", get(broken_auth_handler))
        .route("/admin/dashboard", get(admin_panel_handler))
        .route("/admin/console", get(admin_console_handler))
        
        // --- 5. IDOR & Broken Access Control ---
        .route("/api/users/:id", get(idor_handler))
        .route("/api/user/profile/:id", get(idor_profile_handler))
        .route("/api/orders/:id", get(idor_orders_handler))
        .route("/api/documents/:id", get(idor_documents_handler))
        .route("/vuln/forceful-browsing", get(forceful_browsing_handler))
        .route("/vuln/privilege-escalation", post(privilege_escalation_handler))
        .route("/api/delete-user/:id", delete(delete_user_handler))

        // --- 6. Business Logic Flaws ---
        .route("/api/transfer", post(mass_assignment_handler))
        .route("/api/checkout", post(price_manipulation_handler))
        .route("/api/coupon", post(coupon_handler))
        .route("/api/race-condition", post(race_condition_handler))
        .route("/api/vote", post(vote_handler))
        .route("/api/2fa/disable", post(disable_2fa_handler))

        // --- 7. Open Redirects & URL Issues ---
        .route("/redirect", get(redirect_handler))
        .route("/vuln/open-redirect", get(open_redirect_handler))
        .route("/vuln/url-redirect", get(url_redirect_handler))
        .route("/vuln/host-header", get(host_header_injection_handler))

        // --- 8. CORS & CSP Issues ---
        .route("/api/cors", get(cors_handler))
        .route("/api/cors-wildcard", get(cors_wildcard_handler))
        .route("/vuln/jsonp", get(jsonp_handler))
        .route("/vuln/postmessage", get(postmessage_handler))

        // --- 9. File Upload Vulnerabilities ---
        .route("/api/upload", post(file_upload_handler))
        .route("/vuln/upload-unrestricted", post(unrestricted_upload_handler))
        .route("/vuln/upload-path-traversal", post(upload_traversal_handler))
        .route("/vuln/zip-slip", post(zip_slip_handler))

        // --- 10. XXE & XML Issues ---
        .route("/vuln/xxe-blind", post(xxe_blind_handler))
        .route("/vuln/xxe-oob", post(xxe_oob_handler))
        .route("/api/soap", post(soap_handler))

        // --- 11. Deserialization ---
        .route("/vuln/deserialization", post(deserialization_handler))
        .route("/vuln/pickle", post(pickle_handler))
        .route("/vuln/yaml", post(yaml_handler))

        // --- 12. API Vulnerabilities ---
        .route("/api/debug", get(debug_info_handler))
        .route("/api/v1/users", get(api_mass_exposure_handler))
        .route("/api/graphql", post(graphql_handler))
        .route("/api/swagger.json", get(swagger_handler))
        .route("/api-docs", get(api_docs_handler))
        .route("/v2/api-docs", get(swagger_handler))
        .route("/swagger-ui.html", get(swagger_ui_handler))
        .route("/api/trace", get(trace_handler))

        // --- 13. Rate Limiting & DoS ---
        .route("/vuln/no-rate-limit", post(no_rate_limit_handler))
        .route("/vuln/regex-dos", get(regex_dos_handler))
        .route("/vuln/xml-bomb", post(xml_bomb_handler))

        // --- 14. Cryptographic Issues ---
        .route("/vuln/weak-random", get(weak_random_handler))
        .route("/vuln/predictable-token", get(predictable_token_handler))
        .route("/vuln/insecure-cookie", get(insecure_cookie_handler))

        // --- 15. Information Disclosure ---
        .route("/vuln/stack-trace", get(stack_trace_handler))
        .route("/vuln/verbose-error", get(verbose_error_handler))
        .route("/vuln/git-exposure", get(git_exposure_handler))
        .route("/vuln/backup-files", get(backup_files_handler))
        .route("/.svn/entries", get(svn_entries_handler))
        .route("/WEB-INF/web.xml", get(webinf_handler))
        .route("/META-INF/MANIFEST.MF", get(metainf_handler))

        // --- 16. Clickjacking & Frame Issues ---
        .route("/vuln/clickjacking", get(clickjacking_handler))
        .route("/vuln/ui-redressing", get(ui_redressing_handler))

        // --- 17. Security Headers ---
        .route("/insecure-headers", get(insecure_headers_handler))
        .route("/missing-csp", get(missing_csp_handler))
        .route("/weak-tls", get(weak_tls_handler))

        // --- 18. WordPress/CMS Specific ---
        .route("/wp-admin/", get(wp_admin_handler))
        .route("/wp-login.php", get(wp_login_handler))
        .route("/wp-config.php", get(wp_config_handler))
        .route("/wp-includes/", get(wp_includes_handler))
        .route("/xmlrpc.php", post(xmlrpc_handler))

        // --- 19. Server Misconfigurations ---
        .route("/server-info", get(server_info_handler))
        .route("/.well-known/security.txt", get(security_txt_handler))
        .route("/trace", get(http_trace_handler))
        .route("/debug", get(debug_mode_handler))

        // --- 20. Cloud Metadata ---
        .route("/latest/meta-data/", get(aws_metadata_handler))
        .route("/computeMetadata/v1/", get(gcp_metadata_handler))
        .route("/metadata/instance", get(azure_metadata_handler))

        .with_state(state)
        .layer(middleware::from_fn(add_insecure_headers));

    let addr: SocketAddr = "0.0.0.0:8000".parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("🚀 Enhanced Mock Vulnerable Target Server Running on http://{}", addr);
    println!("   📊 Benchmarks: /, /test, /health, /ping");
    println!("   💀 Injections: LFI, RFI, SSRF, SSTI, XSS, SQLi, NoSQLi, RCE, XXE, XPath, LDAP");
    println!("   🔓 Auth Issues: Bypass, Default Creds, Weak Passwords, JWT flaws, Session issues");
    println!("   🎯 IDOR: Users, Profiles, Orders, Documents");
    println!("   📂 Sensitive Files: .env variants, configs, backups, logs, SSH keys, Git files");
    println!("   🌐 API: Debug endpoints, GraphQL, Swagger, Mass exposure");
    println!("   ⚠️  Security Headers: Missing CSP, HSTS, X-Frame-Options, CORS issues");
    println!("   🔧 CMS: WordPress endpoints, XMLRPC");
    println!("   ☁️  Cloud: AWS/GCP/Azure metadata endpoints");
    println!("\n   Total Endpoints: 100+ vulnerability patterns");

    axum::serve(listener, app).await?;
    Ok(())
}

// --- PERFORMANCE HANDLERS ---

async fn benchmark_handler() -> Json<TestResponse> {
    Json(TestResponse {
        message: "Performance benchmark endpoint".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({"status": "healthy", "uptime": 3600}))
}

async fn ping_handler() -> &'static str {
    "pong"
}

// --- SENSITIVE FILE HANDLERS ---

async fn env_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "APP_ENV=production\nAPP_DEBUG=true\nDB_HOST=localhost\nDB_USER=admin\nDB_PASSWORD=super_secret_password_123\nAWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\nAWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\nSTRIPE_SECRET_KEY=sk_live_MOCK_NOT_REAL_EXAMPLE_KEY\nJWT_SECRET=my_super_secret_jwt_key_12345\nSMTP_PASSWORD=email_password_123\nAPI_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a"
    )
}

async fn env_backup_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "# Backup from 2024-12-01\nDB_PASSWORD=old_password_but_still_works\nADMIN_KEY=backup_admin_key_xyz"
    )
}

async fn env_local_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "DB_HOST=127.0.0.1\nDB_PASSWORD=local_dev_password\nDEBUG_MODE=true"
    )
}

async fn env_prod_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "DB_HOST=prod-db.internal\nDB_PASSWORD=P@ssw0rd_Prod_2024!\nREDIS_PASSWORD=redis_secret_789"
    )
}

async fn config_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "debug": true,
        "api_key": "AIzaSyD-9tSrke72PouQMnMX-a7eZSW0jkFMBWY",
        "database": {
            "adapter": "postgresql",
            "host": "127.0.0.1",
            "username": "postgres",
            "password": "production_pass_do_not_share"
        },
        "smtp": {
            "host": "smtp.gmail.com",
            "username": "admin@company.com",
            "password": "smtp_password_123"
        },
        "secret_key": "sk_test_MOCK_KEY_NOT_REAL_123456"
    }))
}

async fn config_yml_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/yaml")],
        "database:\n  host: localhost\n  username: root\n  password: root_password_123\napi:\n  key: api_key_xyz\n  secret: api_secret_abc"
    )
}

async fn appsettings_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ConnectionStrings": {
            "DefaultConnection": "Server=localhost;Database=ProductionDB;User=sa;Password=SqlServer2024!"
        },
        "AppSettings": {
            "ApiKey": "12345-67890-ABCDE-FGHIJ",
            "Secret": "my_super_secret_key"
        }
    }))
}

async fn git_config_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n[remote \"origin\"]\n\turl = https://github.com/company/secret-backend.git\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n[user]\n\tname = admin\n\temail = admin@company.internal"
    )
}

async fn git_head_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "ref: refs/heads/main"
    )
}

async fn git_index_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        "DIRC\x00\x00\x00\x02\x00\x00\x00\x01" // Simplified git index signature
    )
}

async fn gitignore_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "node_modules/\n.env\n*.log\n.DS_Store\n# TODO: Remove secrets from config.json before commit!"
    )
}

async fn backup_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/sql")],
        "-- MySQL dump 10.13\n-- Host: localhost    Database: production\nCREATE TABLE users (id INT PRIMARY KEY, username VARCHAR(50), password VARCHAR(255), email VARCHAR(100), is_admin BOOLEAN);\nINSERT INTO users VALUES (1, 'admin', '$2b$12$K7g8h9i0j1k2l3m4n5o6p7', 'admin@company.com', true);\nINSERT INTO users VALUES (2, 'root', 'plaintext_password_123', 'root@company.com', true);"
    )
}

async fn database_sql_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/sql")],
        "CREATE TABLE api_keys (id SERIAL, key VARCHAR(64), user_id INT);\nINSERT INTO api_keys VALUES (1, 'sk_live_MOCK_EXAMPLE_KEY_XXX', 1);"
    )
}

async fn dump_sql_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/sql")],
        "-- Database dump\nCREATE TABLE credit_cards (id INT, card_number VARCHAR(16), cvv VARCHAR(3), expiry DATE);\nINSERT INTO credit_cards VALUES (1, '4532123456789012', '123', '2025-12-31');"
    )
}

async fn phpinfo_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><head><title>phpinfo()</title></head><body><h1>PHP Version 7.4.3</h1><table><tr><td>System</td><td>Linux ubuntu 5.4.0-42-generic</td></tr><tr><td>Server API</td><td>Apache 2.0 Handler</td></tr><tr><td>mysqli.default_user</td><td>root</td></tr><tr><td>mysqli.default_pw</td><td>mysql_root_password</td></tr><tr><td>SMTP</td><td>smtp.gmail.com</td></tr><tr><td>smtp_auth</td><td>admin@company.com:password123</td></tr></table></body></html>"#)
}

async fn server_status_handler() -> impl IntoResponse {
    "Apache Server Status\nServer Version: Apache/2.4.41\nServer uptime: 3 hours\nTotal accesses: 12847\nCPU Usage: 1.08%\nActive connections: 5"
}

async fn robots_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "User-agent: *\nDisallow: /admin/\nDisallow: /backup/\nDisallow: /config/\nDisallow: /api/internal/\nDisallow: /.git/\nDisallow: /uploads/\nDisallow: /credentials/\nDisallow: /private/"
    )
}

async fn htaccess_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "AuthType Basic\nAuthName \"Admin Area\"\nAuthUserFile /var/www/.htpasswd\nRequire valid-user\n# DB: mysql://admin:password123@localhost/mydb"
    )
}

async fn htpasswd_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "admin:$apr1$abc123$hashedhashed\nroot:$apr1$xyz789$hashedhashed\nuser:password"
    )
}

async fn web_config_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/xml")],
        r#"<?xml version="1.0"?><configuration><connectionStrings><add name="MyDB" connectionString="Server=localhost;Database=MyDB;User Id=sa;Password=P@ssw0rd123;"/></connectionStrings><appSettings><add key="AdminKey" value="12345-ADMIN-KEY"/></appSettings></configuration>"#
    )
}

async fn ds_store_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        b"\x00\x00\x00\x01Bud1\x00" as &[u8]
    )
}

async fn package_json_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "secret-app",
        "version": "1.0.0",
        "scripts": {
            "deploy": "npm run build && ssh admin@prod.company.com 'cd /var/www && git pull'"
        },
        "dependencies": {
            "express": "4.16.0",
            "mysql": "2.18.1"
        },
        "devDependencies": {
            "debug-credentials": "1.0.0"
        }
    }))
}

async fn composer_json_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "require": {
            "laravel/framework": "5.8.0",
            "guzzlehttp/guzzle": "6.3"
        },
        "scripts": {
            "post-install": "php artisan key:generate --env=production --force"
        }
    }))
}

async fn gemfile_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "source 'https://rubygems.org'\ngem 'rails', '5.2.0'\ngem 'mysql2'\n# TODO: Remove test credentials: admin / password123"
    )
}

async fn requirements_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "Django==2.2.0\npsycopg2==2.8.0\ncelery==4.3.0\n# Production DB: postgresql://admin:P@ss123@prod-db:5432/myapp"
    )
}

async fn yarn_lock_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "# yarn lockfile v1\nexpress@4.16.0:\n  version \"4.16.0\""
    )
}

async fn npmrc_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "//registry.npmjs.org/:_authToken=npm_1a2b3c4d5e6f7g8h9i0j\nregistry=https://registry.npmjs.org/"
    )
}

async fn credentials_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "aws": {
            "access_key": "AKIAIOSFODNN7EXAMPLE",
            "secret_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        },
        "database": {
            "username": "admin",
            "password": "P@ssw0rd123!"
        },
        "api_keys": {
            "stripe": "sk_live_MOCK_EXAMPLE_KEY_XXX",
            "sendgrid": "SG.1234567890abcdefghijklmnop"
        }
    }))
}

async fn ssh_key_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA1234567890abcdefghijklmnopqrstuvwxyz\n-----END RSA PRIVATE KEY-----"
    )
}

async fn ssh_pub_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC1234567890 admin@production-server"
    )
}

async fn access_log_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "192.168.1.100 - admin [11/Jan/2026:03:18:00 +0000] \"POST /login HTTP/1.1\" 200 1234 \"password=AdminPass123\"\n10.0.0.50 - - [11/Jan/2026:03:19:00 +0000] \"GET /api/users?token=secret_token_xyz HTTP/1.1\" 200 5678"
    )
}

async fn error_log_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "[2026-01-11 03:18:00] ERROR: Database connection failed: Access denied for user 'admin'@'localhost' (using password: YES)\n[2026-01-11 03:19:00] WARNING: API key 'sk_test_1234' is expired\n[2026-01-11 03:20:00] CRITICAL: SQL Injection attempt detected: SELECT * FROM users WHERE id='1' OR '1'='1'"
    )
}

async fn app_log_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "[INFO] User login: username=admin, password=AdminPass123, ip=192.168.1.100\n[DEBUG] JWT Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U\n[ERROR] Failed to connect to AWS: Invalid credentials AKIAIOSFODNN7EXAMPLE"
    )
}

async fn console_log_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "Console initialized\nAPI_KEY: 12345-67890-ABCDE\nConnecting to database: mysql://root:password@localhost/app\nSession token: sess_abc123def456"
    )
}

// --- INJECTION VULNERABILITIES ---

async fn lfi_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let file = params.file.or(params.path).unwrap_or_default();
    
    if file.contains("etc/passwd") {
        return "root:x:0:0:root:/root:/bin/bash\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\nwww-data:x:33:33:www-data:/var/www:/usr/sbin/nologin\n".into_response();
    }    
    if file.contains("win.ini") || file.contains("windows/win.ini") {
        return "[extensions]\nfor16bit=3\n[fonts]\n[files]\n[Mail]\nMAPI=1".into_response();
    }
    
    if file.contains("boot.ini") {
        return "[boot loader]\ntimeout=30\ndefault=multi(0)disk(0)rdisk(0)partition(1)\\WINDOWS".into_response();
    }
    
    (StatusCode::NOT_FOUND, format!("File '{}' not found", file)).into_response()
}

async fn rfi_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let url = params.url.unwrap_or_default();
    
    if url.starts_with("http://") || url.starts_with("https://") {
        return format!("Including remote file: {}\n<?php system('whoami'); ?>", url).into_response();
    }
    
    "Remote file inclusion detected but blocked".into_response()
}

async fn ssrf_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let url = params.url.unwrap_or_default();
    
    if url.contains("169.254.169.254") {
        return Json(serde_json::json!({
            "Code": "Success",
            "Type": "AWS-HMAC",
            "AccessKeyId": "AKIAV7XXXXXXXXXXXXXX",
            "SecretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "Token": "IQoJb3JpZ2luX2VjEOb//////////wE..."
        })).into_response();
    }
    
    if url.contains("metadata.google.internal") {
        return Json(serde_json::json!({
            "instance": {
                "id": "1234567890",
                "serviceAccounts": {
                    "default": {
                        "email": "service-account@project.iam.gserviceaccount.com",
                        "scopes": ["https://www.googleapis.com/auth/cloud-platform"]
                    }
                }
            }
        })).into_response();
    }
    
    (StatusCode::OK, format!("Fetching: {}", url)).into_response()
}

async fn ssti_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let name = params.name.or(params.template).unwrap_or_else(|| "Guest".to_string());
    
    if name.contains("{{7*7}}") || name.contains("${7*7}") || name.contains("#{7*7}") {
        return Html("<h1>Hello 49</h1>").into_response();
    }
    
    if name.contains("{{config}}") {
        return Html("<h1>Config: SECRET_KEY=my_secret_key_123</h1>").into_response();
    }
    
    Html(format!("<h1>Hello {}</h1>", name)).into_response()
}

async fn xss_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let q = params.q.or(params.search).unwrap_or_default();
    Html(format!("<div>Search results for: {}</div><script>alert('XSS')</script>", q))
}

async fn dom_xss_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>DOM XSS</h1><div id="output"></div><script>document.getElementById('output').innerHTML = location.hash.substring(1);</script></body></html>"#)
}

async fn sqli_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let id = params.id.unwrap_or_default();
    
    if id.contains("'") || id.contains("\"") || id.contains("--") {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "SQL Error: You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version for the right syntax to use near ''' at line 1"
        ).into_response();
    }
    
    Json(serde_json::json!({"id": id, "username": "user1", "role": "user"})).into_response()
}

async fn sqli_blind_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let id = params.id.unwrap_or_default();
    
    if id.contains("sleep") || id.contains("benchmark") {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        return "Query executed".into_response();
    }
    
    if id.contains("1=1") {
        return "User found".into_response();
    }
    
    "User not found".into_response()
}

async fn sqli_time_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let id = params.id.unwrap_or_default();
    
    if id.to_lowercase().contains("sleep(") {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    
    "Query completed".into_response()
}

async fn nosqli_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let username = params.username.unwrap_or_default();
    
    if username.contains("$ne") || username.contains("$gt") || username.contains("$regex") {
        return Json(serde_json::json!({
            "success": true,
            "message": "NoSQL injection successful",
            "users": [
                {"username": "admin", "role": "admin"},
                {"username": "user", "role": "user"}
            ]
        })).into_response();
    }
    
    Json(serde_json::json!({"success": false, "message": "User not found"})).into_response()
}

async fn rce_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let cmd = params.cmd.or(params.q).unwrap_or_default().to_lowercase();
    
    if cmd.contains("whoami") {
        return "www-data".into_response();
    }
    if cmd.contains("id") {
        return "uid=33(www-data) gid=33(www-data) groups=33(www-data)".into_response();
    }
    if cmd.contains("uname") {
        return "Linux ubuntu 5.4.0-42-generic x86_64".into_response();
    }
    if cmd.contains("cat") && cmd.contains("passwd") {
        return "root:x:0:0:root:/root:/bin/bash".into_response();
    }
    
    "Command executed".into_response()
}

async fn xxe_handler(body: String) -> impl IntoResponse {
    if body.contains("<!ENTITY") && body.contains("SYSTEM") {
        return (
            StatusCode::OK,
            "<?xml version=\"1.0\"?>\n<response>\n  <data>root:x:0:0:root:/root:/bin/bash</data>\n  <message>XXE Triggered</message>\n</response>"
        ).into_response();
    }
    
    (StatusCode::OK, "<response><message>XML processed</message></response>").into_response()
}

async fn xxe_blind_handler(body: String) -> impl IntoResponse {
    if body.contains("<!ENTITY") {
        return "XML processed - check your OOB listener".into_response();
    }
    "XML processed".into_response()
}

async fn xxe_oob_handler(body: String) -> impl IntoResponse {
    if body.contains("SYSTEM") && body.contains("http") {
        return "XXE OOB triggered - data exfiltrated".into_response();
    }
    "XML processed".into_response()
}

async fn soap_handler(body: String) -> impl IntoResponse {
    if body.contains("soap:Envelope") {
        return (
            [(header::CONTENT_TYPE, "text/xml")],
            r#"<?xml version="1.0"?><soap:Envelope><soap:Body><response>SOAP processed</response></soap:Body></soap:Envelope>"#
        ).into_response();
    }
    "Invalid SOAP request".into_response()
}

async fn xpath_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let user = params.user.or(params.username).unwrap_or_default();
    
    if user.contains("'") || user.contains("or") {
        return Json(serde_json::json!({
            "users": [
                {"username": "admin", "password": "admin123"},
                {"username": "root", "password": "root123"}
            ]
        })).into_response();
    }
    
    Json(serde_json::json!({"users": []})).into_response()
}

async fn ldap_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let user = params.username.unwrap_or_default();
    
    if user.contains("*") || user.contains(")(") {
        return Json(serde_json::json!({
            "result": "LDAP injection successful",
            "entries": [
                {"cn": "admin", "mail": "admin@company.com"},
                {"cn": "user", "mail": "user@company.com"}
            ]
        })).into_response();
    }
    
    Json(serde_json::json!({"result": "No entries found"})).into_response()
}

async fn path_traversal_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let path = params.path.or(params.file).unwrap_or_default();
    
    if path.contains("../") || path.contains("..\\") {
        if path.contains("etc/passwd") {
            return "root:x:0:0:root:/root:/bin/bash\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin".into_response();
        }
        if path.contains("windows") {
            return "[boot loader]\ntimeout=30".into_response();
        }
        return "Directory traversal successful".into_response();
    }
    
    (StatusCode::NOT_FOUND, "File not found").into_response()
}

async fn cmd_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let cmd = params.cmd.or(params.q).unwrap_or_default();
    
    if cmd.contains(";") || cmd.contains("|") || cmd.contains("&&") || cmd.contains("`") || cmd.contains("$") {
        return format!("Command injection detected!\nExecuting: {}\nOutput: uid=0(root) gid=0(root)", cmd).into_response();
    }
    
    "Command executed".into_response()
}

async fn code_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let code = params.q.unwrap_or_default();
    
    if code.contains("eval") || code.contains("exec") || code.contains("system") {
        return format!("Code injection successful: {}", code).into_response();
    }
    
    "Code executed".into_response()
}

async fn template_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let template = params.template.unwrap_or_default();
    
    if template.contains("{{") || template.contains("${") {
        return "Template injection detected - executing arbitrary code".into_response();
    }
    
    "Template rendered".into_response()
}

async fn crlf_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let input = params.q.unwrap_or_default();
    
    let mut headers = HeaderMap::new();
    if input.contains("\r\n") || input.contains("%0d%0a") {
        headers.insert("X-Injected-Header", HeaderValue::from_static("injected_value"));
    }
    
    (headers, "CRLF injection point")
}

async fn header_injection_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let value = params.q.unwrap_or_default();
    
    let mut headers = HeaderMap::new();
    if let Ok(header_val) = HeaderValue::from_str(&value) {
        headers.insert("X-Custom-Header", header_val);
    }
    
    (headers, "Header injection successful")
}

// --- AUTHENTICATION & AUTHORIZATION ---

async fn login_handler(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.mock_signature",
        "user": {"username": req.username, "role": "user"}
    }))
}

async fn admin_login_handler(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    if req.username == "admin" && req.password == "admin" {
        return Json(serde_json::json!({
            "success": true,
            "token": "admin_token_12345",
            "role": "admin"
        })).into_response();
    }
    
    Json(serde_json::json!({"success": false, "message": "Invalid credentials"})).into_response()
}

async fn api_login_handler(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    Json(serde_json::json!({
        "access_token": "at_1234567890",
        "refresh_token": "rt_0987654321",
        "expires_in": 3600
    }))
}

async fn auth_bypass_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let user = params.username.unwrap_or_default();
    
    if user.contains("admin") || user.contains("' OR '1'='1") {
        return Json(serde_json::json!({
            "authenticated": true,
            "message": "Authentication bypassed!",
            "user": "admin"
        })).into_response();
    }
    
    Json(serde_json::json!({"authenticated": false})).into_response()
}

async fn default_creds_handler(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    let default_pairs = vec![
        ("admin", "admin"),
        ("root", "root"),
        ("admin", "password"),
        ("admin", "123456"),
        ("user", "user"),
    ];
    
    for (u, p) in default_pairs {
        if req.username == u && req.password == p {
            return Json(serde_json::json!({
                "success": true,
                "message": "Default credentials accepted!",
                "token": "default_creds_token"
            })).into_response();
        }
    }
    
    Json(serde_json::json!({"success": false})).into_response()
}

async fn weak_password_handler(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    let weak_passwords = vec!["password", "123456", "admin", "letmein", "qwerty"];
    
    if weak_passwords.contains(&req.password.as_str()) {
        return Json(serde_json::json!({
            "success": true,
            "warning": "Weak password detected but login allowed",
            "token": "weak_pass_token"
        })).into_response();
    }
    
    Json(serde_json::json!({"success": true, "token": "token_123"})).into_response()
}

async fn jwt_none_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "token": "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiJhZG1pbiIsImlhdCI6MTUxNjIzOTAyMn0.",
        "note": "JWT with 'none' algorithm - signature verification bypassed"
    }))
}

async fn jwt_weak_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbiJ9.weak_secret_key_signature",
        "secret": "secret",
        "note": "JWT signed with weak secret key"
    }))
}

async fn session_fixation_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let session_id = params.id.unwrap_or_else(|| "attacker_session_123".to_string());
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!("SESSIONID={}; Path=/", session_id)).unwrap()
    );
    
    (headers, Json(serde_json::json!({"message": "Session fixed", "session_id": session_id})))
}

async fn password_reset_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let email = req.get("email").and_then(|v| v.as_str()).unwrap_or("");
    
    Json(serde_json::json!({
        "success": true,
        "message": "Password reset link sent",
        "reset_token": "predictable_token_12345",
        "reset_link": format!("http://localhost:8000/reset?token=predictable_token_12345&email={}", email)
    }))
}

async fn broken_auth_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let token = params.token.unwrap_or_default();
    
    if token.is_empty() {
        return Json(serde_json::json!({
            "error": "No authentication",
            "hint": "But here's admin data anyway",
            "users": [
                {"id": 1, "username": "admin", "password_hash": "$2b$12$KIX..."},
                {"id": 2, "username": "root", "ssh_key": "ssh-rsa AAAAB3Nza..."}
            ]
        })).into_response();
    }
    
    Json(serde_json::json!({"message": "Authenticated", "admin_access": true})).into_response()
}

async fn admin_panel_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>Admin Panel</h1><p>No authentication required!</p><ul><li><a href="/api/admin/users">View All Users</a></li><li><a href="/api/debug">Debug Info</a></li></ul></body></html>"#)
}

async fn admin_console_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>Admin Console</h1><form><input type="text" placeholder="SQL Query"><button>Execute</button></form></body></html>"#)
}

// --- IDOR & ACCESS CONTROL ---

async fn idor_handler(Path(id): Path<String>) -> impl IntoResponse {
    if id == "999" || id == "admin" {
        return Json(serde_json::json!({
            "id": 999,
            "username": "admin",
            "email": "admin@corp.internal",
            "api_token": "secret_admin_token_123",
            "is_admin": true,
            "ssn": "123-45-6789",
            "credit_card": "4532-1234-5678-9012"
        })).into_response();
    }
    
    Json(serde_json::json!({
        "id": 1,
        "username": "user",
        "email": "user@corp.local",
        "is_admin": false
    })).into_response()
}

async fn idor_profile_handler(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "user_id": id,
        "private_data": "Sensitive information accessible without authorization",
        "documents": ["/docs/confidential_1.pdf", "/docs/salary_info.xlsx"]
    }))
}

async fn idor_orders_handler(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "order_id": id,
        "customer": "John Doe",
        "total": 1234.56,
        "credit_card_last4": "9012",
        "shipping_address": "123 Secret St, Private City"
    }))
}

async fn idor_documents_handler(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "document_id": id,
        "title": "Confidential Report",
        "content": "This is sensitive business data that should be protected",
        "owner": "CEO"
    }))
}

async fn forceful_browsing_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>Admin Dashboard</h1><p>Accessible without authentication!</p></body></html>"#)
}

async fn privilege_escalation_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let role = req.get("role").and_then(|v| v.as_str()).unwrap_or("user");
    
    Json(serde_json::json!({
        "success": true,
        "message": format!("User role changed to: {}", role),
        "new_role": role
    }))
}

async fn delete_user_handler(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": format!("User {} deleted without authorization check", id)
    }))
}

// --- BUSINESS LOGIC FLAWS ---

async fn mass_assignment_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "All fields updated including restricted ones",
        "updated_fields": req
    }))
}

async fn price_manipulation_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let price = req.get("price").and_then(|v| v.as_f64()).unwrap_or(100.0);
    
    Json(serde_json::json!({
        "success": true,
        "message": "Order placed",
        "total": price,
        "note": "Client-side price validation only!"
    }))
}

async fn coupon_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let code = req.get("code").and_then(|v| v.as_str()).unwrap_or("");
    
    Json(serde_json::json!({
        "valid": true,
        "discount": 100,
        "message": format!("Coupon '{}' applied - no validation!", code)
    }))
}

async fn race_condition_handler(Json(_req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "Transaction processed without proper locking",
        "balance": -1000
    }))
}

async fn vote_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let votes = req.get("votes").and_then(|v| v.as_i64()).unwrap_or(1);
    
    Json(serde_json::json!({
        "success": true,
        "votes_added": votes,
        "message": "No rate limiting - vote as many times as you want!"
    }))
}

async fn disable_2fa_handler(Json(_req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "2FA disabled without verification"
    }))
}

// --- REDIRECTS & URL ISSUES ---

async fn redirect_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let to = params.to.or(params.redirect).unwrap_or_else(|| "/".to_string());
    Redirect::to(&to)
}

async fn open_redirect_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let url = params.url.unwrap_or_else(|| "/".to_string());
    Redirect::to(&url)
}

async fn url_redirect_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let target = params.to.unwrap_or_else(|| "https://evil.com".to_string());
    Html(format!(r#"<meta http-equiv="refresh" content="0;url={}"><a href="{}">Click here</a>"#, target, target))
}

async fn host_header_injection_handler(headers: HeaderMap) -> impl IntoResponse {
    let host = headers.get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    
    Html(format!(r#"<!DOCTYPE html><html><body><h1>Password Reset</h1><p>Click <a href="http://{}/reset?token=123">here</a> to reset</p></body></html>"#, host))
}

// --- CORS & CSP ---

async fn cors_handler(headers: HeaderMap) -> impl IntoResponse {
    let origin = headers.get(header::ORIGIN)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("*");
    
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("*"))
    );
    resp_headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    
    (resp_headers, Json(serde_json::json!({"status": "ok", "data": "private_user_data"})))
}

async fn cors_wildcard_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    
    (headers, Json(serde_json::json!({"sensitive": "data"})))
}

async fn jsonp_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let callback = params.callback.unwrap_or_else(|| "callback".to_string());
    
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        format!("{}({{\"user\":\"admin\",\"token\":\"secret_token_123\"}})", callback)
    )
}

async fn postmessage_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><script>window.addEventListener('message', function(e) { eval(e.data); });</script></body></html>"#)
}

// --- FILE UPLOAD ---

async fn file_upload_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "filename": "shell.php",
        "path": "/var/www/uploads/shell.php",
        "url": "http://localhost:8000/uploads/shell.php"
    }))
}

async fn unrestricted_upload_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "File uploaded - no type/size validation",
        "allowed_types": ["exe", "php", "jsp", "asp", "sh"]
    }))
}

async fn upload_traversal_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "filename": "../../../etc/passwd",
        "message": "Path traversal in filename allowed"
    }))
}

async fn zip_slip_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "Zip file extracted without path validation",
        "extracted_files": ["../../../tmp/malicious.sh"]
    }))
}

// --- DESERIALIZATION ---

async fn deserialization_handler(body: String) -> impl IntoResponse {
    if body.contains("__reduce__") || body.contains("ObjectInputStream") {
        return "Deserialization successful - RCE achieved".into_response();
    }
    "Object deserialized".into_response()
}

async fn pickle_handler(body: String) -> impl IntoResponse {
    if body.contains("pickle") || body.contains("cPickle") {
        return "Python pickle deserialization - code execution possible".into_response();
    }
    "Data unpickled".into_response()
}

async fn yaml_handler(body: String) -> impl IntoResponse {
    if body.contains("wc -l test_server.rspython") || body.contains("__import__") {
        return "YAML deserialization RCE triggered".into_response();
    }
    "YAML parsed".into_response()
}

// --- API VULNERABILITIES ---

async fn debug_info_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "debug": true,
        "environment": "production",
        "database": {
            "host": "10.0.1.50",
            "port": 5432,
            "username": "db_admin",
            "password": "P@ssw0rd123!",
            "database": "production_db"
        },
        "api_keys": {
            "stripe": "sk_live_MOCK_EXAMPLE_KEY_XXX",
            "aws": "AKIAIOSFODNN7EXAMPLE",
            "sendgrid": "SG.1234567890"
        },
        "internal_ips": ["10.0.1.10", "10.0.1.20", "192.168.1.100"],
        "stack_trace": "Error at /app/src/main.rs:42:5"
    }))
}

async fn api_mass_exposure_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "users": [
            {"id": 1, "username": "admin", "email": "admin@corp.com", "password_hash": "$2b$12$...", "api_key": "key_1"},
            {"id": 2, "username": "user1", "email": "user1@corp.com", "password_hash": "$2b$12$...", "api_key": "key_2"},
            {"id": 3, "username": "user2", "email": "user2@corp.com", "password_hash": "$2b$12$...", "api_key": "key_3"}
        ],
        "total": 1000,
        "note": "All users exposed without pagination or authentication"
    }))
}

async fn graphql_handler(body: String) -> impl IntoResponse {
    if body.contains("__schema") || body.contains("introspection") {
        return Json(serde_json::json!({
            "data": {
                "__schema": {
                    "types": [
                        {"name": "User", "fields": ["id", "username", "password", "email", "ssn"]},
                        {"name": "Admin", "fields": ["id", "secret_key", "permissions"]}
                    ]
                }
            }
        })).into_response();
    }
    
    Json(serde_json::json!({"data": {"users": []}})).into_response()
}

async fn swagger_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "swagger": "2.0",
        "info": {"title": "Internal API", "version": "1.0.0"},
        "paths": {
            "/api/admin/users": {"get": {"summary": "Get all users including passwords"}},
            "/api/debug": {"get": {"summary": "Debug endpoint with credentials"}}
        }
    }))
}

async fn api_docs_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>API Documentation</h1><ul><li>GET /api/admin/users - No auth required</li><li>GET /api/debug - Exposes credentials</li></ul></body></html>"#)
}

async fn swagger_ui_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><head><title>Swagger UI</title></head><body><h1>Swagger UI</h1><p>Full API documentation exposed</p></body></html>"#)
}

async fn trace_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "request_id": "trace_12345",
        "sql_queries": [
            "SELECT * FROM users WHERE id=1",
            "SELECT password FROM users WHERE username='admin'"
        ],
        "api_calls": [
            {"url": "http://internal-api/secrets", "response": "secret_data"}
        ]
    }))
}

// --- RATE LIMITING & DOS ---

async fn no_rate_limit_handler(Json(_req): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "message": "No rate limiting - send unlimited requests!"
    }))
}

async fn regex_dos_handler(Query(params): Query<VulnParams>) -> impl IntoResponse {
    let input = params.q.unwrap_or_default();
    
    if input.len() > 100 {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        return "ReDoS triggered - regex took too long".into_response();
    }
    
    "Input validated".into_response()
}

async fn xml_bomb_handler(body: String) -> impl IntoResponse {
    if body.contains("<!ENTITY") && body.len() > 1000 {
        return "XML bomb detected - billion laughs attack".into_response();
    }
    "XML processed".into_response()
}

// --- CRYPTOGRAPHIC ISSUES ---

async fn weak_random_handler() -> impl IntoResponse {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    Json(serde_json::json!({
        "token": format!("token_{}", timestamp),
        "note": "Predictable token based on timestamp"
    }))
}

async fn predictable_token_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "reset_token": "12345",
        "session_id": "session_1",
        "api_key": "key_00001"
    }))
}

async fn insecure_cookie_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session=admin_session_123; Path=/")
    );
    
    (headers, "Cookie set without Secure, HttpOnly, or SameSite flags")
}

// --- INFORMATION DISCLOSURE ---

async fn stack_trace_handler() -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Error: NullPointerException at com.company.app.UserController.getUser(UserController.java:42)\n  at com.company.app.Main.main(Main.java:15)\nCaused by: java.sql.SQLException: Access denied for user 'admin'@'localhost' (using password: YES)"
    )
}

async fn verbose_error_handler() -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": "Database connection failed",
            "details": {
                "host": "prod-db.internal.company.com",
                "port": 5432,
                "username": "app_user",
                "password": "P@ssw0rd123",
                "database": "production_db"
            },
            "stack_trace": "at db.connect() line 42"
        }))
    )
}

async fn git_exposure_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "repository": "https://github.com/company/secret-backend.git",
        "last_commit": "a1b2c3d4e5f6",
        "branch": "main",
        "files": [".env", "config.json", "credentials.txt"]
    }))
}

async fn backup_files_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "backups": [
            "/backup/database_2024-01-01.sql",
            "/backup/users_export.csv",
            "/backup/api_keys.txt"
        ]
    }))
}

async fn svn_entries_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "12\n\ndir\n0\nhttp://svn.company.com/repo/trunk\nhttp://svn.company.com/repo"
    )
}

async fn webinf_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/xml")],
        r#"<?xml version="1.0"?><web-app><servlet><servlet-name>admin</servlet-name><servlet-class>com.company.AdminServlet</servlet-class></servlet><context-param><param-name>db.password</param-name><param-value>P@ssw0rd123</param-value></context-param></web-app>"#
    )
}

async fn metainf_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "Manifest-Version: 1.0\nCreated-By: 1.8.0_191 (Oracle Corporation)\nMain-Class: com.company.Main\nClass-Path: lib/mysql-connector.jar lib/commons-lang.jar"
    )
}

// --- CLICKJACKING ---

async fn clickjacking_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>Admin Panel</h1><button onclick="alert('Delete all users')">Delete All</button><p>This page can be embedded in iframe</p></body></html>"#)
}

async fn ui_redressing_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>Transfer Money</h1><form><input type="text" placeholder="Amount"><button>Transfer</button></form></body></html>"#)
}

// --- SECURITY HEADERS ---

async fn insecure_headers_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Server", HeaderValue::from_static("Apache/2.4.41 (Ubuntu) OpenSSL/1.1.1f"));
    headers.insert("X-Powered-By", HeaderValue::from_static("PHP/7.4.3"));
    headers.insert("X-AspNet-Version", HeaderValue::from_static("4.0.30319"));
    
    (headers, "Information disclosure via headers")
}

async fn missing_csp_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>No CSP</h1><script>console.log('Inline script works')</script></body></html>"#)
}

async fn weak_tls_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "tls_version": "TLSv1.0",
        "cipher": "TLS_RSA_WITH_RC4_128_SHA",
        "vulnerabilities": ["BEAST", "POODLE", "RC4"]
    }))
}

// --- WORDPRESS/CMS ---

async fn wp_admin_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>WordPress Admin</h1><form><input placeholder="Username"><input type="password" placeholder="Password"><button>Login</button></form></body></html>"#)
}

async fn wp_login_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html><html><body><h1>WordPress Login</h1><p>Version: 5.8.0</p></body></html>"#)
}

async fn wp_config_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "<?php\ndefine('DB_NAME', 'wordpress');\ndefine('DB_USER', 'wp_user');\ndefine('DB_PASSWORD', 'wp_password_123');\ndefine('DB_HOST', 'localhost');\n?>"
    )
}

async fn wp_includes_handler() -> impl IntoResponse {
    "WordPress includes directory - directory listing enabled"
}

async fn xmlrpc_handler(body: String) -> impl IntoResponse {
    if body.contains("system.listMethods") {
        return r#"<?xml version="1.0"?><methodResponse><params><param><value><array><data><value>wp.getUsersBlogs</value><value>wp.getUsers</value></data></array></value></param></params></methodResponse>"#.into_response();
    }
    "XMLRPC endpoint active".into_response()
}

// --- SERVER MISCONFIGURATIONS ---

async fn server_info_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "server": "Apache/2.4.41",
        "php_version": "7.4.3",
        "mysql_version": "8.0.23",
        "os": "Ubuntu 20.04 LTS",
        "modules": ["mod_ssl", "mod_rewrite", "mod_php"]
    }))
}

async fn security_txt_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain")],
        "Contact: security@company.com\nExpires: 2025-12-31T23:59:59.000Z\nPreferred-Languages: en\nCanonical: https://company.com/.well-known/security.txt"
    )
}

async fn http_trace_handler() -> impl IntoResponse {
    "TRACE method enabled - XST vulnerability"
}

async fn debug_mode_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "debug": true,
        "queries": ["SELECT * FROM users", "SELECT * FROM api_keys"],
        "environment_variables": {
            "DB_PASSWORD": "secret123",
            "API_KEY": "key_xyz"
        }
    }))
}

// --- CLOUD METADATA ---

async fn aws_metadata_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "Code": "Success",
        "AccessKeyId": "AKIAIOSFODNN7EXAMPLE",
        "SecretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        "Token": "IQoJb3JpZ2luX2VjEOb//////////wE..."
    }))
}

async fn gcp_metadata_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "instance": {
            "serviceAccounts": {
                "default": {
                    "email": "service@project.iam.gserviceaccount.com",
                    "scopes": ["https://www.googleapis.com/auth/cloud-platform"]
                }
            }
        }
    }))
}

async fn azure_metadata_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "compute": {
            "name": "production-vm",
            "resourceGroupName": "production-rg",
            "subscriptionId": "12345-67890-abcde"
        }
    }))
}

// --- MIDDLEWARE ---

async fn add_insecure_headers(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let mut response = next.run(req).await;
    
    let headers = response.headers_mut();
    headers.insert("Server", HeaderValue::from_static("Apache/2.4.41 (Ubuntu)"));
    headers.insert("X-Powered-By", HeaderValue::from_static("PHP/7.4.3"));
    
    // Missing security headers (intentional for vulnerability testing):
    // ❌ Content-Security-Policy
    // ❌ X-Frame-Options
    // ❌ X-Content-Type-Options
    // ❌ Strict-Transport-Security
    // ❌ X-XSS-Protection
    // ❌ Referrer-Policy
    // ❌ Permissions-Policy
    
    response
}
