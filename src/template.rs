use minijinja::{Environment, context};

use crate::models::Session;

pub fn format_text(
    template: &str,
    session: &Session,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut env = Environment::new();

    env.add_template("discord", template)?;

    let template = env.get_template("discord")?;

    let result = template.render(context! {
        minecraft_version => session.instance.minecraft_version,
        instance_name => session.instance.name,
        profile_name => session.instance.name
    })?;

    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Instance;
    #[test]
    fn supports_documented_and_existing_instance_variables() {
        let session = Session {
            instance: Instance {
                name: "My instance".into(),
                minecraft_version: "1.21.1".into(),
            },
            started_at: 1,
        };
        assert_eq!(
            format_text(
                "{{ profile_name }} / {{ instance_name }} / {{ minecraft_version }}",
                &session
            )
            .unwrap(),
            "My instance / My instance / 1.21.1"
        );
    }
}
