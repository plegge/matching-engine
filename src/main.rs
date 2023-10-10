use tonic::{transport::Server, Request, Response, Status};
use matchingengine_v1::matching_engine_server::{MatchingEngine,MatchingEngineServer};
use matchingengine_v1::*;
use std::env;
use std::collections::HashMap;
use std::sync::Mutex;
use core::fmt::Debug;
use uuid::Uuid;
use std::time::SystemTime;


pub mod matchingengine_v1 {
    tonic::include_proto!("matchingengine_v1");
}

pub trait OrderRepository: Send + Sync{
    fn place_order(&self, order: Order) -> Result<Order, &'static str>;
    fn cancel_order(&self, order_id: String) -> Result<(), &'static str>;
    fn get_order_status(&self, order_id: String) -> Result<Order, &'static str>;
    fn get_all_orders(&self, user_id: String) -> Result<Vec<Order>, &'static str>;
}

impl Debug for dyn OrderRepository {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OrderRepository{{{}}}", "_")
    }
}

#[derive(Debug, Default)]
pub struct InMemoryOrderRepository {
    orders: Mutex<HashMap<String, Order>>,
}

// Implement Default manually for EngineServer
impl Default for EngineServer {
    fn default() -> Self {
        Self {
            repository: Box::new(InMemoryOrderRepository::new())
        }
    }
}

// Define a new method for InMemoryOrderRepository if it doesn't exist
impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self {
            orders: Mutex::new(HashMap::new())
        }
    }
}

impl OrderRepository for InMemoryOrderRepository {
    fn place_order(&self, order: Order) -> Result<Order, &'static str> {
        let mut orders = self.orders.lock().unwrap();
        orders.insert(order.id.clone(), order.clone());
        Ok(order)
    }

    fn cancel_order(&self, order_id: String) -> Result<(), &'static str> {
        let mut orders = self.orders.lock().unwrap();
        orders.remove(&order_id);
        Ok(())
    }

    fn get_order_status(&self, order_id: String) -> Result<Order, &'static str> {
        let orders = self.orders.lock().unwrap();
        orders.get(&order_id).cloned().ok_or("order not found")
    }

    fn get_all_orders(&self, user_id: String) -> Result<Vec<Order>, &'static str> {
        let orders = self.orders.lock().unwrap();
        Ok(orders.values()
            .filter(|&order| order.user_id == user_id)
            .cloned()
            .collect())
    }
}


#[derive(Debug)]
pub struct EngineServer{
    repository: Box<dyn OrderRepository>,
}

impl EngineServer {
    pub fn new(repository: Box<dyn OrderRepository>) -> Self {
        Self { repository }
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
            Ok(orders) => Ok(Response::new(matchingengine_v1::ListOrdersResponse{
                orders: orders.clone(),
            })),
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
            status: matchingengine_v1::OrderStatus::Created.into(),
            create_time: Some(SystemTime::now().into()),
            update_time: None,
            cancel_time: None,
        };

        match self.repository.place_order(order) {
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
    let addr = env::var("GRPC_ADDR").unwrap_or_else(|_| "[::1]:50051".to_string()).parse()?;

    let repository: Box<dyn OrderRepository> = Box::new(InMemoryOrderRepository::new());

    let engine = EngineServer::new(repository);

    println!("starting server at {}", addr);
    Server::builder()
            .add_service(MatchingEngineServer::new(engine))
            .serve(addr)
            .await?;

    Ok(())
}

