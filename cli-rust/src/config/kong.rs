use crate::app::App;
use std::fs;
use std::path::Path;

pub fn generate(app: &App, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let gw_dir = output_dir.join("docker").join("gateway");
    fs::create_dir_all(&gw_dir)?;

    let mut routes: Vec<String> = Vec::new();
    for &svc in &app.enabled_services {
        let docker_name = svc.docker_name();
        let port = svc.port();
        let prefix = svc.route_path();
        routes.push(format!("{prefix}=http://{docker_name}:{port}"));
    }

    let routes_override = routes.join(";");

    let mut env_out = String::new();
    env_out.push_str("# Gateway route configuration\n");
    env_out.push_str("# These override the default routes built into gateway-rust\n");
    env_out.push_str(&format!("ROUTES_OVERRIDE={routes_override}\n"));
    env_out.push_str(&format!("CORS_ORIGIN={}\n", app.cors_origins));
    if app.uses_redis() {
        env_out.push_str(&format!(
            "REDIS_URL=redis://{}:{}\n",
            app.redis_host, app.redis_port
        ));
    }

    let env_path = gw_dir.join("gateway.env");
    fs::write(&env_path, &env_out)?;

    let mut compose_out = String::new();
    compose_out.push_str("# Gateway service overrides for docker-compose\n");
    compose_out.push_str("  gateway-svc:\n");
    compose_out.push_str("    environment:\n");
    compose_out.push_str(&format!("      ROUTES_OVERRIDE: \"{routes_override}\"\n"));
    compose_out.push_str(&format!("      CORS_ORIGIN: \"{}\"\n", app.cors_origins));

    let compose_path = gw_dir.join("gateway-overrides.yml");
    fs::write(&compose_path, &compose_out)?;

    Ok(())
}
