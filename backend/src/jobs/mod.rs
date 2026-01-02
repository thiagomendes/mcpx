//! Background Jobs Module
//!
//! Jobs in this module are designed to be extractable to separate services.
//! They are stateless and communicate only through the database.

pub mod alert_evaluator;
