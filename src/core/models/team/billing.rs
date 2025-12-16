//! Team billing models

use serde::{Deserialize, Serialize};

/// Team billing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBilling {
    /// Billing plan
    pub plan: BillingPlan,
    /// Billing status
    pub status: BillingStatus,
    /// Monthly budget limit
    pub monthly_budget: Option<f64>,
    /// Current month usage
    pub current_usage: f64,
    /// Billing cycle start
    pub cycle_start: chrono::DateTime<chrono::Utc>,
    /// Billing cycle end
    pub cycle_end: chrono::DateTime<chrono::Utc>,
    /// Payment method
    pub payment_method: Option<PaymentMethod>,
    /// Billing address
    pub billing_address: Option<BillingAddress>,
}

/// Billing plan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPlan {
    /// Free plan
    Free,
    /// Starter plan
    Starter,
    /// Professional plan
    Professional,
    /// Enterprise plan
    Enterprise,
    /// Custom plan
    Custom,
}

/// Billing status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingStatus {
    /// Active billing status
    Active,
    /// Past due billing status
    PastDue,
    /// Cancelled billing status
    Cancelled,
    /// Suspended billing status
    Suspended,
    /// Trial billing status
    Trial,
}

/// Payment method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    /// Payment method type
    pub method_type: PaymentMethodType,
    /// Last 4 digits (for cards)
    pub last_four: Option<String>,
    /// Expiry month (for cards)
    pub expiry_month: Option<u32>,
    /// Expiry year (for cards)
    pub expiry_year: Option<u32>,
    /// Brand (for cards)
    pub brand: Option<String>,
}

/// Payment method type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    /// Credit card payment
    CreditCard,
    /// Debit card payment
    DebitCard,
    /// Bank transfer payment
    BankTransfer,
    /// PayPal payment
    PayPal,
    /// Stripe payment
    Stripe,
}

/// Billing address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    /// Company name
    pub company: Option<String>,
    /// Address line 1
    pub line1: String,
    /// Address line 2
    pub line2: Option<String>,
    /// City
    pub city: String,
    /// State/Province
    pub state: Option<String>,
    /// Postal code
    pub postal_code: String,
    /// Country
    pub country: String,
    /// Tax ID
    pub tax_id: Option<String>,
}
