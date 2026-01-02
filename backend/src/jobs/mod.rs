//! Background Jobs Module
//!
//! Jobs in this module are designed to run in the separate worker service.
//! They are stateless and communicate only through the database.

pub mod alert_evaluator;
pub mod handlers;
