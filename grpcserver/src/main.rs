use tonic::{transport::Server, Request, Response, Status};
use protos::matching_engine_server::{MatchingEngine,MatchingEngineServer};
use protos::{Order, ListOrdersRequest, ListOrdersResponse, PlaceOrderRequest, PlaceOrderResponse, OrderStatus, CancelOrderRequest, CancelOrderResponse, GetOrderStatusRequest, GetOrderStatusResponse};
use std::env;
use core::fmt::Debug;
use uuid::Uuid;
use std::time::SystemTime;
use repository::{OrderRepository, InMemoryOrderRepository};
use producer::{OrderProducer, KafkaProducer};

// Implementing the Server
#[derive(Debug)]
pub struct EngineServer{
    repository: Box<dyn OrderRepository>,
    producer: Box<dyn OrderProducer>,
}

impl EngineServer {
    pub fn new(repository: Box<dyn OrderRepository>, producer: Box<dyn OrderProducer>) -> Self {
        Self { repository, producer }
    }
}

impl Default for EngineServer {
    fn default() -> Self {
        Self {
            repository: Box::new(InMemoryOrderRepository::new()),
            producer: Box::new(KafkaProducer::new()),
        }
    }
}

#[tonic::async_trait]
impl MatchingEngine for EngineServer {
    async fn list_orders(
        &self,
        request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        let list_request = request.into_inner();
        let orders = self.repository.get_all_orders(list_request.user_id).clone();

        match orders {
            Ok(orders) => Ok(Response::new(ListOrdersResponse{
                orders: orders.clone(),
            })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_order_status(
        &self,
        request: Request<GetOrderStatusRequest>,
    ) -> Result<Response<GetOrderStatusResponse>, Status> {
        let get_request = request.into_inner();
        let order = self.repository.get_order_status(get_request.id).clone();

        match order {
            Ok(order) => Ok(Response::new(GetOrderStatusResponse { order: Some(order.clone()) })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        let order_request = request.into_inner();

        let order = Order {
            id: Uuid::new_v4().to_string(),  // Generating a new UUID for the order
            user_id: order_request.user_id.clone(),
            pair: order_request.pair.clone(),
            price: order_request.price,
            quantity: order_request.quantity,
            r#type: order_request.r#type.clone(),
            status: OrderStatus::Created.into(),
            create_time: Some(SystemTime::now().into()),
            update_time: None,
            cancel_time: None,
        };

        match self.producer.place_order(order) {
            Ok(order) => {
                let response = PlaceOrderResponse {
                    id: order.id,
                    status: order.status,
                };
                Ok(Response::new(response))
            },
            Err(e) => {
                Err(Status::internal(e))
            }
        }
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
        let cancel_request = request.into_inner();

        match self.producer.cancel_order(cancel_request.id) {
            Ok(_) => {
                Ok(Response::new(CancelOrderResponse { status: protos::OrderStatus::Cancelled.into() }))
            },
            Err(e) => {
                Err(Status::internal(e))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("initializing...");
    let addr = env::var("GRPC_ADDR").unwrap_or_else(|_| "[::1]:50051".to_string()).parse()?;

    let repo: Box<dyn OrderRepository> = Box::new(InMemoryOrderRepository::new());
    let prod: Box<dyn OrderProducer> = Box::new(KafkaProducer::new());

    let engine = EngineServer::new(repo, prod);

    println!("starting server at {}", addr);
    Server::builder()
            .add_service(MatchingEngineServer::new(engine))
            .serve(addr)
            .await?;

    Ok(())
}

