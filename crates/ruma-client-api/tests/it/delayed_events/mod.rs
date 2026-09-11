#[cfg(feature = "client")]
use std::borrow::Cow;

#[cfg(feature = "client")]
use http::Request as HttpRequest;
#[cfg(feature = "client")]
use ruma_common::api::{
    FeatureFlag, MatrixVersion, OutgoingRequest, OutgoingRequestExt as _, SupportedVersions,
    auth_scheme::{AccessToken, SendAccessToken},
    error::IntoHttpError,
    path_builder::VersionHistory,
};

mod finalized;
#[cfg(feature = "server")]
mod incoming;
#[cfg(feature = "client")]
mod legacy;
#[cfg(feature = "client")]
mod outgoing;
#[cfg(any(feature = "client", feature = "server"))]
mod response;

#[cfg(feature = "client")]
fn outgoing<R>(
    request: R,
    feature: FeatureFlag,
    token: SendAccessToken<'_>,
) -> Result<HttpRequest<Vec<u8>>, IntoHttpError>
where
    R: OutgoingRequest<Authentication = AccessToken, PathBuilder = VersionHistory>,
{
    request.try_into_http_request("https://example.org", token, versions(feature))
}

#[cfg(feature = "client")]
fn versions(feature: FeatureFlag) -> Cow<'static, SupportedVersions> {
    Cow::Owned(SupportedVersions {
        versions: [MatrixVersion::V1_19].into(),
        features: [feature].into(),
    })
}
