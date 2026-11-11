use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum PopupState {
    Loading { original: String },
    Ready { original: String, corrected: String },
    NoMistakes { original: String },
    Failed { message: String },
}

impl PopupState {
    pub fn from_correction(original: &str, corrected: &str) -> Self {
        if original.trim() == corrected.trim() {
            return Self::NoMistakes {
                original: original.to_string(),
            };
        }
        Self::Ready {
            original: original.to_string(),
            corrected: corrected.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_different_correction_is_ready_to_accept() {
        let state = PopupState::from_correction("i has a dog", "I have a dog");
        assert_eq!(
            state,
            PopupState::Ready {
                original: "i has a dog".to_string(),
                corrected: "I have a dog".to_string()
            }
        );
    }

    #[test]
    fn a_correction_equal_to_the_original_means_no_mistakes() {
        let state = PopupState::from_correction("I have a dog.", "I have a dog.\n");
        assert!(matches!(state, PopupState::NoMistakes { .. }));
    }
}
