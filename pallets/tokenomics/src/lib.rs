//! # Sanctuary Tokenomics Pallet
//!
//! This pallet implements the core economic model of Sanctuary Protocol as defined
//! in the Yellow Paper (Chapters 2 & 3).
//!
//! ## Key Features
//!
//! 1. **Sigmoid Emission Curve**: Token supply follows a logistic (S-curve) function
//!    instead of Bitcoin's harsh step function. This creates organic growth patterns.
//!
//! 2. **Adaptive Scarcity Mechanism (ASM)**: The "Time Dilation" concept where
//!    economic time runs faster when network activity is high, accelerating scarcity.
//!
//! 3. **Universal Constants**: Supply derived from π × e × φ × 10^6 = 13,817,422 SANC
//!
//! ## Mathematical Background
//!
//! ### Cumulative Supply Function
//! ```text
//! S(t) = S_max / (1 + e^(-k(t - t_0)))
//! ```
//! Where:
//! - S_max = 13,817,422 SANC (maximum supply)
//! - k = growth rate constant
//! - t_0 = inflection point (when 50% supply is emitted)
//! - t = effective block height (dilated by network activity)
//!
//! ### Block Reward (derivative of S(t))
//! ```text
//! R_b(t) = k × S(t) × (1 - S(t)/S_max)
//! ```
//! This produces a bell curve - rewards start low, peak mid-adoption, then decline.
//!
//! ### Time Dilation (ASM)
//! ```text
//! t_eff(n) = t_eff(n-1) + Δt × (1 + α × (Q_net - Q_baseline))
//! ```
//! High network activity (Q_net) accelerates economic time, making supply scarcer.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

/// Fixed-point math module for on-chain exponential calculations
pub mod math;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, Get, OnUnbalanced},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{Saturating, Zero},
        Perbill,
    };

    /// The balance type of this pallet.
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Negative imbalance (tokens created from thin air)
    pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    // ═══════════════════════════════════════════════════════════════════════════
    // SANCTUARY CONSTANTS (from Yellow Paper)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Precision for fixed-point arithmetic (10^12 for better precision)
    pub const PRECISION: u128 = 1_000_000_000_000;

    /// π × 10^9 = 3,141,592,653
    pub const PI: u128 = 3_141_592_653;

    /// e × 10^9 = 2,718,281,828
    pub const E: u128 = 2_718_281_828;

    /// φ × 10^9 = 1,618,033,988
    pub const PHI: u128 = 1_618_033_988;

    /// Maximum supply in smallest units (13,817,422 × 10^18)
    pub const MAX_SUPPLY: u128 = 13_817_422_000_000_000_000_000_000;

    /// Maximum supply in whole units (13,817,422)
    pub const MAX_SUPPLY_UNITS: u128 = 13_817_422;

    /// One token unit (10^18)
    pub const SANC: u128 = 1_000_000_000_000_000_000;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait for the tokenomics pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type for token operations.
        type Currency: Currency<Self::AccountId>;

        /// Handler for newly minted tokens (e.g., distribute to validators).
        type OnTokensMinted: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Treasury account that receives a portion of minted tokens.
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// Percentage of block reward that goes to treasury (e.g., 10%).
        #[pallet::constant]
        type TreasuryCut: Get<Perbill>;

        /// Number of blocks per era (for ASM updates).
        #[pallet::constant]
        type BlocksPerEra: Get<BlockNumberFor<Self>>;

        /// Growth rate constant (k) for sigmoid curve.
        /// Represented as k × 10^12 for precision.
        #[pallet::constant]
        type GrowthRateK: Get<u128>;

        /// Inflection point (t_0) - block number when 50% supply is emitted.
        #[pallet::constant]
        type InflectionPoint: Get<BlockNumberFor<Self>>;

        /// Sensitivity constant (α) for time dilation.
        /// Represented as α × 10^12 for precision.
        #[pallet::constant]
        type TimeDilationAlpha: Get<u128>;

        /// Baseline network activity quotient.
        #[pallet::constant]
        type BaselineActivity: Get<u128>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════════════════

    /// Total tokens minted so far (cumulative supply).
    #[pallet::storage]
    #[pallet::getter(fn total_minted)]
    pub type TotalMinted<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Effective block height (time-dilated).
    /// This can be greater than actual block height if network activity is high.
    #[pallet::storage]
    #[pallet::getter(fn effective_block_height)]
    pub type EffectiveBlockHeight<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Network Activity Quotient (Q_net) for the current era.
    /// Higher values indicate more network activity.
    #[pallet::storage]
    #[pallet::getter(fn network_activity)]
    pub type NetworkActivity<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Block number of the last era update.
    #[pallet::storage]
    #[pallet::getter(fn last_era_block)]
    pub type LastEraBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Transaction volume in current era (for Q_net calculation).
    #[pallet::storage]
    #[pallet::getter(fn era_transaction_volume)]
    pub type EraTransactionVolume<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Number of active accounts in current era.
    #[pallet::storage]
    #[pallet::getter(fn era_active_accounts)]
    pub type EraActiveAccounts<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Total fees burned in current era.
    #[pallet::storage]
    #[pallet::getter(fn era_fees_burned)]
    pub type EraFeesBurned<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // ═══════════════════════════════════════════════════════════════════════════
    // GENESIS CONFIG
    // ═══════════════════════════════════════════════════════════════════════════

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Initial supply already minted at genesis.
        pub initial_supply: BalanceOf<T>,
        /// Starting effective block height.
        pub initial_effective_height: u128,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            TotalMinted::<T>::put(self.initial_supply);
            EffectiveBlockHeight::<T>::put(self.initial_effective_height);
            NetworkActivity::<T>::put(T::BaselineActivity::get());
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tokens were minted as block reward.
        BlockRewardMinted {
            block_number: BlockNumberFor<T>,
            effective_height: u128,
            reward: BalanceOf<T>,
            total_supply: BalanceOf<T>,
        },

        /// Network activity quotient was updated.
        NetworkActivityUpdated {
            era: BlockNumberFor<T>,
            q_net: u128,
            time_dilation_factor: u128,
        },

        /// Era ended and metrics were reset.
        EraEnded {
            era: BlockNumberFor<T>,
            transaction_volume: BalanceOf<T>,
            active_accounts: u32,
            fees_burned: BalanceOf<T>,
        },

        /// Maximum supply reached - no more minting.
        MaxSupplyReached {
            total_supply: BalanceOf<T>,
        },
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════════════════

    #[pallet::error]
    pub enum Error<T> {
        /// Maximum supply has been reached.
        MaxSupplyReached,
        /// Arithmetic overflow in calculation.
        ArithmeticOverflow,
        /// Era update not yet due.
        EraNotEnded,
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HOOKS
    // ═══════════════════════════════════════════════════════════════════════════

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Called at the end of each block.
        /// Handles:
        /// 1. Updating effective block height (with time dilation)
        /// 2. Calculating and distributing block rewards
        /// 3. Era transitions (updating Q_net)
        fn on_finalize(block_number: BlockNumberFor<T>) {
            // Update effective block height with time dilation
            Self::update_effective_height();

            // Calculate and mint block reward
            if let Ok(reward) = Self::calculate_and_mint_reward(block_number) {
                if !reward.is_zero() {
                    let total = Self::total_minted();
                    Self::deposit_event(Event::BlockRewardMinted {
                        block_number,
                        effective_height: Self::effective_block_height(),
                        reward,
                        total_supply: total,
                    });
                }
            }

            // Check if era ended and update network activity
            let last_era = Self::last_era_block();
            let blocks_per_era: u32 = T::BlocksPerEra::get()
                .try_into()
                .unwrap_or(14400u32); // Default: 1 day at 6s blocks
            let current_block: u32 = block_number
                .try_into()
                .unwrap_or(0u32);
            let last_block: u32 = last_era
                .try_into()
                .unwrap_or(0u32);

            if current_block.saturating_sub(last_block) >= blocks_per_era {
                Self::end_era(block_number);
            }
        }

        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // Weight for on_finalize operations
            T::WeightInfo::on_finalize()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTRINSICS
    // ═══════════════════════════════════════════════════════════════════════════

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Records a transaction for network activity tracking.
        /// This is called by the transaction payment pallet or other modules.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::record_transaction())]
        pub fn record_transaction(
            origin: OriginFor<T>,
            volume: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            EraTransactionVolume::<T>::mutate(|v| *v = v.saturating_add(volume));
            EraActiveAccounts::<T>::mutate(|a| *a = a.saturating_add(1));

            Ok(())
        }

        /// Records fees burned for Q_net calculation.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::record_fee_burn())]
        pub fn record_fee_burn(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            EraFeesBurned::<T>::mutate(|f| *f = f.saturating_add(amount));

            Ok(())
        }

        /// Force an era transition (sudo only, for testing).
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::force_era_end())]
        pub fn force_era_end(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;

            let block = frame_system::Pallet::<T>::block_number();
            Self::end_era(block);

            Ok(())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // INTERNAL FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    impl<T: Config> Pallet<T> {
        /// Calculate the sigmoid supply at a given effective block height.
        ///
        /// S(t) = S_max / (1 + e^(-k(t - t_0)))
        ///
        /// Uses Taylor series approximation for e^x.
        pub fn sigmoid_supply(effective_height: u128) -> u128 {
            let k = T::GrowthRateK::get(); // k × 10^12
            let t0: u128 = T::InflectionPoint::get()
                .try_into()
                .unwrap_or(55_296_000u128);

            // Calculate exponent: -k(t - t0)
            // Note: We handle the sign separately
            let (exponent, is_negative) = if effective_height >= t0 {
                // t > t0: exponent is negative (we want e^(-k(t-t0)))
                let delta = effective_height.saturating_sub(t0);
                let exp_val = delta.saturating_mul(k) / PRECISION;
                (exp_val, true)
            } else {
                // t < t0: exponent is positive (we're before inflection)
                let delta = t0.saturating_sub(effective_height);
                let exp_val = delta.saturating_mul(k) / PRECISION;
                (exp_val, false)
            };

            // Calculate e^exponent using Taylor series
            let e_power = crate::math::exp_approximation(exponent);

            // Calculate denominator: 1 + e^(-k(t-t0))
            let denominator = if is_negative {
                // e^(-x) = 1/e^x for positive x
                // denominator = 1 + 1/e^x = (e^x + 1) / e^x
                // But we need to handle this carefully in fixed point
                // For large x, e^(-x) ≈ 0, so denominator ≈ 1
                // For small x, we compute properly
                if e_power > PRECISION * 1000 {
                    // e^x is very large, so e^(-x) ≈ 0
                    PRECISION // denominator ≈ 1
                } else if e_power == 0 {
                    PRECISION * 2 // e^x = 1, so denominator = 2
                } else {
                    PRECISION + PRECISION.saturating_mul(PRECISION) / e_power
                }
            } else {
                // denominator = 1 + e^x
                PRECISION + e_power
            };

            // S(t) = S_max / denominator
            if denominator == 0 {
                return MAX_SUPPLY;
            }

            MAX_SUPPLY.saturating_mul(PRECISION) / denominator
        }

        /// Calculate block reward as derivative of sigmoid.
        ///
        /// R_b(t) = k × S(t) × (1 - S(t)/S_max)
        ///
        /// This produces a bell curve - low at start, peaks mid-adoption, then declines.
        pub fn calculate_block_reward(effective_height: u128) -> u128 {
            let k = T::GrowthRateK::get();
            let current_supply = Self::sigmoid_supply(effective_height);

            // S(t) / S_max (as fraction × PRECISION)
            let supply_ratio = current_supply.saturating_mul(PRECISION) / MAX_SUPPLY;

            // 1 - S(t)/S_max
            let remaining_ratio = PRECISION.saturating_sub(supply_ratio);

            // k × S(t) × (1 - S(t)/S_max) / PRECISION^2
            // But we need to scale properly for block reward
            // The raw derivative would be tiny, so we scale it appropriately
            let reward_raw = k
                .saturating_mul(current_supply)
                .saturating_mul(remaining_ratio)
                / PRECISION
                / PRECISION;

            // Scale to reasonable block reward (divide by blocks per year roughly)
            // This gives us annual emission / blocks_per_year
            reward_raw / 5_259_600 // blocks per year at 6s blocks
        }

        /// Calculate and mint block reward.
        fn calculate_and_mint_reward(
            _block_number: BlockNumberFor<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let effective_height = Self::effective_block_height();
            let current_minted = Self::total_minted();

            // Check if we've reached max supply
            let max_supply_balance: BalanceOf<T> = MAX_SUPPLY
                .try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow)?;

            if current_minted >= max_supply_balance {
                Self::deposit_event(Event::MaxSupplyReached {
                    total_supply: current_minted,
                });
                return Ok(Zero::zero());
            }

            // Calculate theoretical block reward
            let reward_u128 = Self::calculate_block_reward(effective_height);

            // Ensure we don't exceed max supply
            let remaining: u128 = max_supply_balance
                .saturating_sub(current_minted)
                .try_into()
                .unwrap_or(0);
            let actual_reward_u128 = reward_u128.min(remaining);

            if actual_reward_u128 == 0 {
                return Ok(Zero::zero());
            }

            let reward: BalanceOf<T> = actual_reward_u128
                .try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow)?;

            // Mint the reward
            let imbalance = T::Currency::issue(reward);

            // Update total minted
            TotalMinted::<T>::mutate(|m| *m = m.saturating_add(reward));

            // Split between treasury and validators
            let treasury_cut = T::TreasuryCut::get();
            let treasury_amount = treasury_cut.mul_floor(reward);
            let _validator_amount = reward.saturating_sub(treasury_amount);

            // Deposit treasury cut
            if !treasury_amount.is_zero() {
                let _ = T::Currency::deposit_creating(&T::TreasuryAccount::get(), treasury_amount);
            }

            // Send validator rewards through the handler
            // For now, just handle the imbalance
            T::OnTokensMinted::on_unbalanced(imbalance);

            Ok(reward)
        }

        /// Update effective block height with time dilation.
        ///
        /// t_eff(n) = t_eff(n-1) + 1 × (1 + α × (Q_net - Q_baseline))
        fn update_effective_height() {
            let current_height = Self::effective_block_height();
            let q_net = Self::network_activity();
            let baseline = T::BaselineActivity::get();
            let alpha = T::TimeDilationAlpha::get();

            // Calculate time dilation factor
            let dilation_factor = if q_net > baseline {
                // Network is busy - accelerate time
                let excess = q_net.saturating_sub(baseline);
                let acceleration = alpha.saturating_mul(excess) / PRECISION;
                PRECISION.saturating_add(acceleration)
            } else {
                // Network is quiet - normal time
                PRECISION
            };

            // New height = old height + 1 × dilation_factor / PRECISION
            let new_height = current_height.saturating_add(dilation_factor / PRECISION);

            // Ensure minimum increment of 1
            let final_height = if new_height == current_height {
                current_height.saturating_add(1)
            } else {
                new_height
            };

            EffectiveBlockHeight::<T>::put(final_height);
        }

        /// End the current era and calculate new Q_net.
        fn end_era(current_block: BlockNumberFor<T>) {
            let tx_volume = Self::era_transaction_volume();
            let active_accounts = Self::era_active_accounts();
            let fees_burned = Self::era_fees_burned();
            let total_supply = Self::total_minted();

            // Calculate Q_net using logarithmic dampening
            // Q_net = w1 × ln(1 + V_active) + w2 × ln(1 + T_vol) + w3 × (B_burned / S_circ)
            // For simplicity, we use a weighted linear combination with sqrt for diminishing returns

            let tx_vol_u128: u128 = tx_volume.try_into().unwrap_or(0);
            let fees_u128: u128 = fees_burned.try_into().unwrap_or(0);
            let supply_u128: u128 = total_supply.try_into().unwrap_or(1);

            // Component 1: Active participation (weight: 40%)
            let participation_score = (active_accounts as u128)
                .saturating_mul(PRECISION)
                .saturating_mul(40)
                / 100;

            // Component 2: Transaction volume (weight: 40%)
            // Use sqrt approximation for diminishing returns
            let volume_score = crate::math::integer_sqrt(tx_vol_u128 / SANC)
                .saturating_mul(PRECISION)
                .saturating_mul(40)
                / 100;

            // Component 3: Fee burn ratio (weight: 20%)
            let burn_ratio = if supply_u128 > 0 {
                fees_u128.saturating_mul(PRECISION) / supply_u128
            } else {
                0
            };
            let burn_score = burn_ratio.saturating_mul(20) / 100;

            // Combine into Q_net
            let q_net = participation_score
                .saturating_add(volume_score)
                .saturating_add(burn_score);

            // Calculate time dilation factor for event
            let baseline = T::BaselineActivity::get();
            let alpha = T::TimeDilationAlpha::get();
            let dilation = if q_net > baseline {
                let excess = q_net.saturating_sub(baseline);
                PRECISION.saturating_add(alpha.saturating_mul(excess) / PRECISION)
            } else {
                PRECISION
            };

            // Emit events
            Self::deposit_event(Event::EraEnded {
                era: current_block,
                transaction_volume: tx_volume,
                active_accounts,
                fees_burned,
            });

            Self::deposit_event(Event::NetworkActivityUpdated {
                era: current_block,
                q_net,
                time_dilation_factor: dilation,
            });

            // Update storage
            NetworkActivity::<T>::put(q_net);
            LastEraBlock::<T>::put(current_block);

            // Reset era metrics
            EraTransactionVolume::<T>::put(BalanceOf::<T>::zero());
            EraActiveAccounts::<T>::put(0u32);
            EraFeesBurned::<T>::put(BalanceOf::<T>::zero());
        }

        // ═══════════════════════════════════════════════════════════════════════
        // PUBLIC GETTERS FOR RUNTIME API
        // ═══════════════════════════════════════════════════════════════════════

        /// Get the theoretical maximum supply.
        pub fn max_supply() -> u128 {
            MAX_SUPPLY
        }

        /// Get current emission progress as percentage (× 100).
        pub fn emission_progress() -> u32 {
            let minted: u128 = Self::total_minted().try_into().unwrap_or(0);
            let progress = minted.saturating_mul(10000) / MAX_SUPPLY;
            progress.min(10000) as u32
        }

        /// Get estimated remaining blocks until 99% emission.
        pub fn blocks_until_near_completion() -> u128 {
            let current_height = Self::effective_block_height();
            let t0: u128 = T::InflectionPoint::get()
                .try_into()
                .unwrap_or(55_296_000u128);

            // 99% emission happens at roughly t0 + 4.6/k
            // For our parameters, this is approximately 2 × t0
            let completion_height = t0.saturating_mul(2);

            if current_height >= completion_height {
                0
            } else {
                completion_height.saturating_sub(current_height)
            }
        }

        /// Get current block reward rate.
        pub fn current_block_reward() -> u128 {
            let effective_height = Self::effective_block_height();
            Self::calculate_block_reward(effective_height)
        }
    }
}
