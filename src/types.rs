use clap::ValueEnum;
use std::fmt;

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum ProjectType {
    Frontend,
    Backend,
    Fullstack,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum Frontend {
    React,
    Angular,
    Vue,
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Runtime {
    Go,
    Node,
    Python,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum ApiFramework {
    Gin,
    Echo,
    Fiber,
    Express,
    Fastify,
    Fastapi,
    Flask,
}

// ── Display (lowercase, matches template folder names & config values) ────────

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no skipped variants")
            .get_name()
            .fmt(f)
    }
}

impl fmt::Display for Frontend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no skipped variants")
            .get_name()
            .fmt(f)
    }
}

impl fmt::Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no skipped variants")
            .get_name()
            .fmt(f)
    }
}

impl fmt::Display for ApiFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no skipped variants")
            .get_name()
            .fmt(f)
    }
}

// ── Domain logic on types ─────────────────────────────────────────────────────

impl Frontend {
    /// npm script name used to start the dev server.
    pub fn dev_script(&self) -> &'static str {
        match self {
            Self::Angular => "start",
            Self::React | Self::Vue => "dev",
        }
    }
}

impl Runtime {
    /// Ordered list of API frameworks available for this runtime.
    pub fn api_frameworks(&self) -> Vec<ApiFramework> {
        match self {
            Self::Go => vec![ApiFramework::Gin, ApiFramework::Echo, ApiFramework::Fiber],
            Self::Node => vec![ApiFramework::Express, ApiFramework::Fastify],
            Self::Python => vec![ApiFramework::Fastapi, ApiFramework::Flask],
        }
    }
}

impl ApiFramework {
    /// The runtime this framework belongs to.
    pub fn runtime(&self) -> Runtime {
        match self {
            Self::Gin | Self::Echo | Self::Fiber => Runtime::Go,
            Self::Express | Self::Fastify => Runtime::Node,
            Self::Fastapi | Self::Flask => Runtime::Python,
        }
    }

    /// Arguments passed to `python3 -m <args>` when running the dev server.
    pub fn python_dev_args(&self) -> Vec<&'static str> {
        match self {
            Self::Fastapi => vec!["uvicorn", "main:app", "--reload"],
            Self::Flask => vec!["flask", "run", "--debug"],
            _ => vec!["main"],
        }
    }
}
