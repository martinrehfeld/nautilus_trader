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

//! Common types, constants, and utilities for the Alpaca adapter.

pub mod consts;
pub mod credential;
pub mod enums;
pub mod models;
pub mod parse;
pub mod urls;

pub use consts::*;
pub use credential::AlpacaCredential;
pub use enums::{
    AlpacaAssetClass, AlpacaDataFeed, AlpacaEnvironment, AlpacaOrderSide, AlpacaOrderType,
    AlpacaTimeInForce,
};
pub use models::*;
pub use parse::*;
pub use urls::{get_http_base_url, get_ws_data_url, get_ws_trading_url};
