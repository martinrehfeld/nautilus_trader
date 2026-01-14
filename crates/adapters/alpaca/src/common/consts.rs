// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2026 Andrew Crum. All rights reserved.
//  https://github.com/agcrum
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Constants for the Alpaca adapter.

use ustr::Ustr;

/// The Alpaca venue identifier used throughout NautilusTrader.
///
/// This venue is used for all Alpaca instruments regardless of asset class.
pub const ALPACA_VENUE: &str = "ALPACA";

/// The NautilusTrader Alpaca broker ID.
///
/// This is used for identifying NautilusTrader clients in API requests.
pub const ALPACA_NAUTILUS_BROKER_ID: &str = "NAUTILUS";

/// Returns the Alpaca venue as a `Ustr`.
///
/// This is the preferred way to get the venue identifier for use with
/// Nautilus types that require `Ustr` venue identifiers.
#[must_use]
pub fn alpaca_venue() -> Ustr {
    Ustr::from(ALPACA_VENUE)
}

/// Default WebSocket heartbeat interval in seconds.
pub const DEFAULT_WS_HEARTBEAT_SECS: u64 = 30;

/// Default HTTP request timeout in seconds.
pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 30;

/// Maximum number of symbols per WebSocket subscription request.
///
/// Alpaca recommends subscribing in batches to avoid overwhelming the server.
pub const MAX_SYMBOLS_PER_SUBSCRIPTION: usize = 100;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_alpaca_venue_constant() {
        assert_eq!(ALPACA_VENUE, "ALPACA");
    }

    #[rstest]
    fn test_alpaca_venue_ustr() {
        let venue = alpaca_venue();
        assert_eq!(venue.as_str(), "ALPACA");
    }
}
