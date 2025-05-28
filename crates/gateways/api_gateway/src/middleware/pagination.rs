use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum PaginationType {
  #[serde(rename = "cursor")]
  Cursor { next_cursor: Option<String>, prev_cursor: Option<String>, has_more: bool },
  #[serde(rename = "offset")]
  Offset {
    current_page: u32,
    per_page: u32,
    total_pages: u32,
    total_items: u64,
    has_next: bool,
    has_prev: bool,
  },
}

#[derive(Debug, Serialize, Clone)]
pub struct PaginationMetadata {
  pub pagination_type: PaginationType,
  pub order_by: Option<String>,
  pub order_direction: Option<String>,
}

impl Default for PaginationMetadata {
  fn default() -> Self {
    Self {
      pagination_type: PaginationType::Offset {
        current_page: 1,
        per_page: 10,
        total_pages: 1,
        total_items: 0,
        has_next: false,
        has_prev: false,
      },
      order_by: None,
      order_direction: None,
    }
  }
}

impl PaginationMetadata {
  pub fn new_cursor(
    next_cursor: Option<String>,
    prev_cursor: Option<String>,
    has_more: bool,
  ) -> Self {
    Self {
      pagination_type: PaginationType::Cursor { next_cursor, prev_cursor, has_more },
      order_by: None,
      order_direction: None,
    }
  }

  pub fn new_offset(
    current_page: u32,
    per_page: u32,
    total_pages: u32,
    total_items: u64,
    has_next: bool,
    has_prev: bool,
  ) -> Self {
    Self {
      pagination_type: PaginationType::Offset {
        current_page,
        per_page,
        total_pages,
        total_items,
        has_next,
        has_prev,
      },
      order_by: None,
      order_direction: None,
    }
  }

  pub fn with_order(mut self, order_by: String, order_direction: String) -> Self {
    self.order_by = Some(order_by);
    self.order_direction = Some(order_direction);
    self
  }
}

#[derive(Debug, Serialize)]
pub struct ResponseMetadata {
  pub timestamp: String,
  pub content_type: String,
  pub pagination: Option<PaginationMetadata>,
}

impl ResponseMetadata {
  pub fn new(timestamp: String, content_type: String) -> Self {
    Self { timestamp, content_type, pagination: None }
  }

  pub fn with_pagination(mut self, pagination: PaginationMetadata) -> Self {
    self.pagination = Some(pagination);
    self
  }
}
