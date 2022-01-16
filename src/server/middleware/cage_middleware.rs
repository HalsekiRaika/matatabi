use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::Error;
use futures::{Future, ready, TryFutureExt};
use futures::future::{ok, Ready};
use tonic::transport::Channel;
use crate::server::middleware::bearer::Credentials;
use crate::server::middleware::cage::cage_api_client::CageApiClient;

pub type CageMiddlewareTaskResult = Result<ServiceRequest, actix_web::Error>;

#[derive(Debug, Clone)]
pub struct CageAuth<F, O>
  where F: FnMut(CageApiClient<Channel>, ServiceRequest, Credentials) -> O,
        O: std::future::Future<Output = CageMiddlewareTaskResult>
{
    validator_client: CageApiClient<Channel>,
    validator: Arc<Mutex<F>>
}
#[derive(Debug, Clone)]
pub struct CageAuthMiddleware<S, F, O>
  where F: FnMut(CageApiClient<Channel>, ServiceRequest, Credentials) -> O,
        O: std::future::Future<Output = CageMiddlewareTaskResult>
{
    service: Rc<RefCell<S>>,
    validator_client: CageApiClient<Channel>,
    validator: Arc<Mutex<F>>
}

impl<F, O> CageAuth<F, O>
  where F: FnMut(CageApiClient<Channel>, ServiceRequest, Credentials) -> O,
        O: std::future::Future<Output = CageMiddlewareTaskResult> {
    pub fn new(validator_client: CageApiClient<Channel>, validator_func: F) -> CageAuth<F, O> {
        Self { validator_client, validator: Arc::new(Mutex::new(validator_func)) }
    }
}

impl<S, B, F, O> Transform<S, ServiceRequest> for CageAuth<F, O>
  where S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
        S::Future: 'static,
        B: MessageBody + 'static,
        B::Error: Into<Error>,
        F: FnMut(CageApiClient<Channel>, ServiceRequest, Credentials) -> O + 'static,
        O: std::future::Future<Output = CageMiddlewareTaskResult> + 'static
{
    //type Request = ServiceRequest;
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Transform = CageAuthMiddleware<S, F, O>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CageAuthMiddleware {
            service: Rc::new(RefCell::new(service)),
            validator_client: self.validator_client.clone(),
            validator: self.validator.clone()
        })
    }
}

impl<S, B, F, O> Service<ServiceRequest> for CageAuthMiddleware<S, F, O>
  where S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
        S::Future: 'static,
        B: MessageBody + 'static,
        B::Error: Into<Error>,
        F: FnMut(CageApiClient<Channel>, ServiceRequest, Credentials) -> O + 'static,
        O: Future<Output = CageMiddlewareTaskResult> + 'static
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;//S::Future;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        #[allow(unused_mut)]
        let mut client = self.validator_client.clone();
        let validator = self.validator.clone();
        Box::pin(async move {
            let (credentials, req) = match BearerExtraction::new(req).await {
                Ok(req) => req,
                Err((err, req)) => {
                    return Ok(req.error_response(err).map_into_right_body());
                }
            };
            let mut validator = validator.lock().unwrap();
            let req = validator(client, req, credentials).await?;
            service.call(req).await
                .map(|res| res.map_into_left_body())
        })
    }
}

struct BearerExtraction {
    req: Option<ServiceRequest>,
    future: Option<Pin<Box<dyn Future<Output = Result<Credentials, actix_web::Error>>>>>
}

impl BearerExtraction {
    fn new(req: ServiceRequest) -> Self {
        Self { req: Some(req), future: None }
    }
}

impl Future for BearerExtraction {
    type Output = Result<(Credentials, ServiceRequest), (Error, ServiceRequest)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.future.is_none() {
            let req = self.req.as_ref()
                .expect("cannot unwrap ServiceRequest optional.");
            let future = Credentials::from_req(req).map_err(Into::into);
            self.future = Some(Box::pin(future));
        }

        let future = self.future.as_mut().expect("cannot fire future.");
        let credentials = ready!(
            future.as_mut().poll(cx)
            .map_err(|e| {
                (e, self.req.take().expect("cannot take ServiceRequest."))
            })
        )?;
        let req = self.req.take().expect("cannot take ServiceRequest.");
        Poll::Ready(Ok((credentials, req)))
    }
}
