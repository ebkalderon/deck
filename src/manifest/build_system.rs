use std::borrow::Cow;
use std::path::Path;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BuildSystem<'m> {
    Cargo {
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        features: Vec<Cow<'m, str>>,
        #[serde(default)]
        default_features: bool,
    },
    CMake {
        generator: CMakeGenerator,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        defines: Vec<(Cow<'m, str>, Cow<'m, str>)>,
    },
    Gnu {
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        configure_flags: Vec<Cow<'m, str>>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        make_flags: Vec<Cow<'m, str>>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        modify_phases: Vec<ModifyPhase<'m>>,
    },
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CMakeGenerator {
    Makefiles,
    Ninja,
    WMake,
    CbNinja,
    CbMakefiles,
    ClNinja,
    ClMakefiles,
    SublimeNinja,
    SublimeMakefiles,
    KateNinja,
    KateMakefiles,
    EclipseNinja,
    EclipseMakefiles,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GnuPhase {
    Unpack,
    Configure,
    Build,
    Check,
    Install,
    Strip,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ModifyPhase<'m> {
    AddAfter {
        phase: GnuPhase,
        #[serde(rename = "do")]
        action: Vec<Operation<'m>>,
    },
    AddBefore {
        phase: GnuPhase,
        #[serde(rename = "do")]
        action: Vec<Operation<'m>>,
    },
    Delete(GnuPhase),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Operation<'m> {
    Println(Cow<'m, str>),
    Replace(Cow<'m, Path>, Cow<'m, str>, Cow<'m, str>),
}
