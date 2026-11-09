pub const FUEL_TYPES: [&str; 6] = ["Petrol", "Diesel", "LPG", "CNG", "Hybrid", "Electric"];
pub const ACQ_KINDS: [&str; 4] = ["Purchase", "Lease", "Rental", "Donation"];
pub const BASE_STATUS: [&str; 2] = ["Active", "Retired"];
pub const VEHICLE_STATUS: [&str; 4] = ["Available", "Assigned", "Maintenance", "Retired"];
pub const LICENSE_CATS: [&str; 11] = ["AM", "A1", "A2", "A", "B", "BE", "C1", "C", "CE", "D", "DE"];
pub const EMPLOYMENT: [&str; 4] = ["Full-time", "Part-time", "Contractor", "Temporary"];
pub const MAINT_KINDS: [&str; 6] = [
    "Routine service",
    "Repair",
    "Tyres",
    "Inspection",
    "Bodywork",
    "Other",
];
pub const MAINT_STATUS: [&str; 3] = ["Scheduled", "In progress", "Completed"];
pub const EXPENSE_CATS: [&str; 6] = ["Toll", "Parking", "Tax", "Repair", "Insurance", "Other"];
pub const DOC_TYPES: [&str; 6] = [
    "Insurance",
    "Inspection",
    "Registration",
    "Driving license",
    "Permit",
    "Other",
];

pub fn nz(id: i64) -> Option<i64> {
    if id == 0 {
        None
    } else {
        Some(id)
    }
}

#[derive(Clone, Default, Debug)]
pub struct Vehicle {
    pub id: i64,
    pub plate: String,
    pub make: String,
    pub model: String,
    pub vin: String,
    pub year: i64,
    pub fuel_type: String,
    pub mileage: i64,
    pub acq_date: String,
    pub acq_price: f64,
    pub acq_kind: String,
    pub base_status: String,
    pub image_path: String,
    pub notes: String,
    pub status: String,
}

#[derive(Clone, Default, Debug)]
pub struct Driver {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub tax_code: String,
    pub birth_date: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub license_number: String,
    pub license_category: String,
    pub license_expiry: String,
    pub hire_date: String,
    pub job_title: String,
    pub employment_type: String,
    pub active: bool,
    pub notes: String,
}

#[derive(Clone, Default, Debug)]
pub struct Assignment {
    pub id: i64,
    pub vehicle_id: i64,
    pub driver_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub start_km: i64,
    pub end_km: Option<i64>,
    pub notes: String,
}

#[derive(Clone, Default, Debug)]
pub struct Maintenance {
    pub id: i64,
    pub vehicle_id: i64,
    pub kind: String,
    pub description: String,
    pub provider: String,
    pub scheduled_date: String,
    pub completed_date: String,
    pub cost: f64,
    pub mileage: i64,
    pub status: String,
    pub next_due_km: i64,
    pub next_due_date: String,
}

#[derive(Clone, Default, Debug)]
pub struct Fuel {
    pub id: i64,
    pub vehicle_id: i64,
    pub driver_id: i64,
    pub date: String,
    pub liters: f64,
    pub price: f64,
    pub mileage: i64,
    pub station: String,
    pub full_tank: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Expense {
    pub id: i64,
    pub vehicle_id: i64,
    pub category: String,
    pub date: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Clone, Default, Debug)]
pub struct Document {
    pub id: i64,
    pub vehicle_id: i64,
    pub driver_id: i64,
    pub doc_type: String,
    pub number: String,
    pub issuer: String,
    pub issue_date: String,
    pub expiry_date: String,
    pub file_path: String,
    pub notes: String,
}

#[derive(Clone, Default, Debug)]
pub struct Activity {
    pub ts: String,
    pub message: String,
}
