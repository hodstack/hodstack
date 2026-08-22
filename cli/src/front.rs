#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Front {
    pub name: String,
    pub description: String,
    pub user: bool,
}

impl Front {
    pub fn read(text: &str, stem: &str) -> Self {
        let mut front = Self {
            name: stem.to_owned(),
            description: String::new(),
            user: false,
        };

        let Some(matter) = matter(text) else {
            return front;
        };

        for line in matter.lines() {
            let Some((key, value)) = field(line) else {
                continue;
            };

            match key {
                "name" => value.clone_into(&mut front.name),
                "description" => value.clone_into(&mut front.description),
                "disable-model-invocation" => front.user = value == "true",
                _ => {}
            }
        }

        front
    }
}

fn matter(text: &str) -> Option<&str> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;

    rest.split_once("\n---").map(|(matter, _)| matter)
}

fn field(line: &str) -> Option<(&str, &str)> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }

    let (key, value) = line.split_once(':')?;

    Some((key.trim(), unquote(value.trim())))
}

fn unquote(value: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = value.strip_prefix(quote) {
            if let Some(inner) = inner.strip_suffix(quote) {
                return inner;
            }
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_front_matter_gives_a_name_and_a_description() {
        let front = Front::read(
            "---\nname: pr-review\ndescription: \"Read a pull request\"\n---\n\n# Body\n",
            "stem",
        );

        assert_eq!(front.name, "pr-review");
        assert_eq!(front.description, "Read a pull request");
        assert!(!front.user);
    }

    #[test]
    fn a_skill_that_the_model_cannot_reach_is_for_the_user() {
        let front = Front::read(
            "---\nname: learn\ndescription: Write a rule\ndisable-model-invocation: true\n---\n",
            "learn",
        );

        assert!(front.user);
    }

    #[test]
    fn a_file_without_front_matter_takes_the_name_of_its_file() {
        let front = Front::read("# A rule\n", "queue-worker-restart");

        assert_eq!(front.name, "queue-worker-restart");
        assert_eq!(front.description, "");
    }

    #[test]
    fn a_nested_field_is_not_a_field_of_the_skill() {
        let front = Front::read("---\nname: learn\nmetadata:\n  name: other\n---\n", "learn");

        assert_eq!(front.name, "learn");
    }
}
