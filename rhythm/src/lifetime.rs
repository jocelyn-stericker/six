use specs::Entity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Why this this element created?
pub enum Lifetime {
    /// This is a rest that was automatically inserted by the program.
    ///
    /// This should be considered empty space, and can be replaced without explicit user intent.
    AutomaticRest,

    /// This is a rest that is hidden.
    ///
    /// Used, for example, in pickup beats.
    HiddenRest,

    /// The user is hovering over a space and this is a preview.
    ///
    /// Moving the cursor will remove this note or rest.
    Temporary(Entity),

    /// The user has added this note.
    ///
    /// Explicit user intent is required to remove it.
    Explicit(Entity),
}

impl Lifetime {
    pub fn is_explicit(&self) -> bool {
        match self {
            Lifetime::Explicit(_) => true,
            _ => false,
        }
    }

    pub fn is_temporary(&self) -> bool {
        match self {
            Lifetime::Temporary(_) => true,
            _ => false,
        }
    }

    pub fn is_automatic(&self) -> bool {
        match self {
            Lifetime::AutomaticRest => true,
            _ => false,
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Lifetime::HiddenRest => true,
            _ => false,
        }
    }

    pub fn to_option(self) -> Option<Entity> {
        match self {
            Lifetime::AutomaticRest | Lifetime::HiddenRest => None,
            Lifetime::Temporary(e) | Lifetime::Explicit(e) => Some(e),
        }
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Lifetime::AutomaticRest
    }
}

