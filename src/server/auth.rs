use tonic::{Request, Response};
use tonic::transport::Channel;
use cage::cage_api_client::CageApiClient;
use cage::{AccountConfig, Account, Token};

pub mod cage { tonic::include_proto!("cage"); }

async fn create_credentials(
    mut client: CageApiClient<Channel>,
       request: Request<AccountConfig>) -> Response<Token> {
    client.create_account(request).await.unwrap()
}

async fn verify_credentials(
    mut client: CageApiClient<Channel>, 
       request: Request<Token>) -> Response<Account> {
    client.verification(request).await.unwrap()
}