pub mod account_info;
pub mod break_policy;
mod error;
pub mod mfa;
pub mod pto;
mod public;
mod session;
pub mod time_entries;

use attohttpc::RequestBuilder;
use attohttpc::Response;
pub use error::Error;
pub use public::Client as PublicClient;
pub use session::Session;

pub type Result<T> = std::result::Result<T, Error>;

fn request_to_result<E, F, T, B>(req: RequestBuilder<B>, f: F) -> Result<T>
where
    F: FnOnce(Response) -> std::result::Result<T, E>,
    Error: From<E>,
    B: attohttpc::body::Body,
{
    let res = req.send()?;
    match res.status() {
        attohttpc::StatusCode::OK => f(res).map_err(Error::from),
        attohttpc::StatusCode::CREATED => f(res).map_err(Error::from),
        _ => Err(res.into()),
    }
}
