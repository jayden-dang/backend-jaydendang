#[macro_export]
macro_rules! ensure_txn {
  ($txn:expr) => {
    if $txn.is_none() {
      return Err(Error::no_txn());
    }
  };
}

#[macro_export]
macro_rules! ensure_txn_not_committed {
  ($state:expr) => {
    if $state.is_committed() {
      return Err(Error::already_committed());
    }
  };
}

#[macro_export]
macro_rules! ensure_txn_enabled {
  ($config:expr) => {
    if !$config.txn_enabled {
      return Err(Error::cannot_begin_txn());
    }
  };
}
