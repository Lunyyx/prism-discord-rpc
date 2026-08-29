use minijinja::{Environment, context};

use crate::models::Session;

pub fn format_text(template: &str, session: &Session) -> Result<String, Box<dyn std::error::Error>>  {
    let mut env = Environment::new();

    env.add_template("discord", template)?;

    let template = env.get_template("discord")?;
    
    let result = template.render(context! {
        minecraft_version => session.instance.minecraft_version,
        instance_name => session.instance.name
    })?;

    Ok(result)
}