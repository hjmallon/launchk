use std::fmt;
use std::fmt::Formatter;

bitflags! {
    #[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
    /// Bitmask for filtering on the job type, which is a mix
    /// of scope (where it's located), and kind (agent v. daemon)
    pub struct JobTypeFilter: u32 {
        const SYSTEM = (1 << 1);
        const GLOBAL = (1 << 2);
        const USER   = (1 << 3);
        const AGENT  = (1 << 4);
        const DAEMON = (1 << 5);
        const LOADED = (1 << 6);
    }
}

impl JobTypeFilter {
    pub fn launchk_default() -> Self {
        let mut jtf = JobTypeFilter::default();
        jtf.toggle(JobTypeFilter::LOADED);
        jtf
    }
}

/// Represent the bitmask as a string for easy TUI check for styling
/// hotkey status
impl fmt::Display for JobTypeFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut display = format!("");

        if (*self & JobTypeFilter::SYSTEM) == JobTypeFilter::SYSTEM {
            display.push('s');
        }

        if (*self & JobTypeFilter::GLOBAL) == JobTypeFilter::GLOBAL {
            display.push('g');
        }

        if (*self & JobTypeFilter::USER) == JobTypeFilter::USER {
            display.push('u');
        }

        if (*self & JobTypeFilter::AGENT) == JobTypeFilter::AGENT {
            display.push('a');
        }

        if (*self & JobTypeFilter::DAEMON) == JobTypeFilter::DAEMON {
            display.push('d');
        }

        if (*self & JobTypeFilter::LOADED) == JobTypeFilter::LOADED {
            display.push('l');
        }

        write!(f, "{}", display)
    }
}

impl fmt::Debug for JobTypeFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            JobTypeFilter::SYSTEM => write!(f, "SYSTEM"),
            JobTypeFilter::GLOBAL => write!(f, "GLOBAL"),
            JobTypeFilter::USER   => write!(f, "USER"),
            JobTypeFilter::AGENT  => write!(f, "AGENT"),
            JobTypeFilter::DAEMON => write!(f, "DAEMON"),
            JobTypeFilter::LOADED => write!(f, "LOADED"),
            _                     => Ok(())
        }
    }
}
