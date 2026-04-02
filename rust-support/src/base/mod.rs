//! Base Module
//!
//! Contains base traits and types for entities, resources, controllers, and services

pub mod entity;
pub mod resource;
pub mod controller;
pub mod service;

// Re-export commonly used types
pub use entity::{BaseEntity, EntityConnection, SerializableEntity};
pub use resource::{
    BaseResource, ViewResource, ShowResource, ResourceCollection, CollectionMeta,
};
pub use controller::{BaseController, IndexQuery};
pub use service::{
    BaseService, ServiceContainer, PaginationParams, SortParams, SortDirection, FilterParams,
    FilterValue,
};
