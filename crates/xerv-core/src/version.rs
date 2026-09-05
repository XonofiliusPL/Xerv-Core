use semver::Version;

/// Core API version. Unstable until `1.0.0`.
pub const API_VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 0,
    pre: semver::Prerelease::EMPTY,
    build: semver::BuildMetadata::EMPTY,
};

#[inline]
pub fn api_version() -> Version {
    API_VERSION
}
