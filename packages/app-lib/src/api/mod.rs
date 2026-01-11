//! API for interacting with Theseus
pub mod cache;
// [AR] Patch: Removed friends module
pub mod handler;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod minecraft_auth;
pub mod minecraft_skins;
// [AR] Patch: Removed mr_auth module
pub mod pack;
pub mod process;
pub mod profile;
pub mod server_address;
pub mod settings;
pub mod tags;
pub mod update; // [AR] Feature
pub mod worlds;

pub mod data {
    pub use crate::state::{
        CacheBehaviour,
        CacheValueType,
        Credentials,
        Dependency,
        DirectoryInfo,
        Hooks,
        JavaVersion,
        LinkedData,
        MemorySettings,
        ModLoader,
        // [AR] Patch: Removed ModrinthCredentials
        Organization,
        ProcessMetadata,
        ProfileFile,
        Project,
        ProjectType,
        SearchResult,
        SearchResults,
        Settings,
        TeamMember,
        Theme,
        User,
        // [AR] Patch: Removed UserFriend export
        Version,
        WindowSize,
    };
    pub use ariadne::users::UserStatus;
}

pub mod prelude {
    pub use crate::{
        State,
        data::*,
        event::CommandPayload,
        jre, metadata, minecraft_auth, /* [AR] Removed: mr_auth, */ pack,
        process,
        profile::{self, Profile, create},
        settings,
        util::{
            io::{IOError, canonicalize},
            network::{is_network_metered, tcp_listen_any_loopback},
        },
    };
}
