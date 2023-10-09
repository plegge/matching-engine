use tonic::{transport::Server, Request, Response, Status};
use matchingengine_v1::matching_engine_server::{MatchingEngine,MatchingEngineServer};
use matchingengine_v1::*;

pub mod matchingengine_v1 {
    tonic::include_proto!("matchengine_v1");
}

#[derive(Debug, Default)]
pub struct EngineServer{}

#[tonic::async_trait]
impl MatchingEngine for EngineServer {
    async fn list_orders(
        &self,
        request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        println!("We got the request: {:?}", request);

        let reply = matchingengine_v1::ListOrdersResponse{
            orders: vec![],
        };

        Ok(Response::new(reply))
    }

    async fn place_order(
        &self,
        _request: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        Err(Status::internal("not implemented"))
    }

    async fn cancel_order(
        &self,
        _request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        Err(Status::internal("not implemented"))
    }

    async fn get_order_status(
        &self,
        _request: Request<GetOrderStatusRequest>,
    ) -> Result<Response<GetOrderStatusResponse>, Status> {
        Err(Status::internal("not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("initializing...");
    let addr = "[::1]:50051".parse()?;
    let engine = EngineServer::default();

    println!("starting server...");
    Server::builder()
            .add_service(MatchingEngineServer::new(engine))
            .serve(addr)
            .await?;

    Ok(())
}
