//! Attention management system for AI agents
//!
//! This module provides functionality for managing an AI agent's attention and response behavior
//! in various contexts. It includes:
//! - Content quality evaluation
//! - Message response decision making
//! - Social media interaction decisions (likes, retweets, quotes)
//!
//! The system uses a combination of token-based filtering and AI-powered evaluation to determine
//! appropriate actions for different types of content.
//!
//! # Key Features
//! - **Content Evaluation**: Assesses the quality and knowledge value of articles and posts
//! - **Response Management**: Determines when to respond, ignore, or stop in conversations
//! - **Social Interaction**: Decides on appropriate social media interactions based on content
//! - **Customizable Parameters**: Allows configuration of token thresholds and stop phrases
//!
//! # Usage
//! The system is designed to be integrated into AI agents that need to manage their attention
//! and interactions in complex environments. It provides both automated decision-making and
//! configurable parameters for fine-tuning behavior.

use anda_core::{evaluate_tokens, AgentOutput, CompletionFeatures, CompletionRequest, Message};

static HIGH_REWARD_COMMAND: &str = "HIGH_REWARD";
static MEDIUM_REWARD_COMMAND: &str = "MEDIUM_REWARD";
static RESPOND_COMMAND: &str = "RESPOND";
static IGNORE_COMMAND: &str = "IGNORE";
static STOP_COMMAND: &str = "STOP";
static STOP_PHRASES: [&str; 13] = [
    "shut up",
    "dont talk",
    "silence",
    "stop talking",
    "be quiet",
    "hush",
    "wtf",
    "stfu",
    "stupid bot",
    "dumb bot",
    "stop responding",
    "can you not",
    "can you stop",
];

/// Enum representing possible attention commands for message handling
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum AttentionCommand {
    /// Stop responding to the conversation
    Stop,
    /// Default state - ignore the message
    #[default]
    Ignore,
    /// Respond to the message
    Respond,
}

/// Enum representing content quality levels
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum ContentQuality {
    /// Content should be ignored
    #[default]
    Ignore,
    /// Content is of good quality
    Good,
    /// Content is exceptional quality
    Exceptional,
}

/// Attention management system for handling message responses and content evaluation
#[derive(Debug, Clone)]
pub struct Attention {
    /// List of phrases that should trigger a stop response
    phrases: Vec<String>,
    /// Minimum number of tokens required for a prompt to be considered
    pub min_prompt_tokens: usize,
    /// Minimum number of tokens required for content to be evaluated
    pub min_content_tokens: usize,
}

impl Default for Attention {
    fn default() -> Self {
        Self {
            phrases: STOP_PHRASES.iter().map(|s| s.to_string()).collect(),
            min_prompt_tokens: 4,
            min_content_tokens: 60,
        }
    }
}

impl Attention {
    /// Creates a new Attention instance with custom parameters
    ///
    /// # Arguments
    /// * `phrases` - List of stop phrases
    /// * `min_prompt_tokens` - Minimum token count for prompt evaluation
    /// * `min_content_tokens` - Minimum token count for content evaluation
    pub fn new(phrases: Vec<String>, min_prompt_tokens: usize, min_content_tokens: usize) -> Self {
        Self {
            phrases,
            min_prompt_tokens,
            min_content_tokens,
        }
    }

    /// Evaluates the quality of content based on knowledge value
    ///
    /// # Arguments
    /// * `ctx` - Completion context implementing CompletionFeatures
    /// * `content` - Content to evaluate
    ///
    /// # Returns
    /// ContentQuality enum indicating the evaluation result
    pub async fn evaluate_content(
        &self,
        ctx: &impl CompletionFeatures,
        content: &str,
    ) -> ContentQuality {
        // Ignore very short content
        if evaluate_tokens(content) < self.min_content_tokens {
            return ContentQuality::Ignore;
        }

        let req = CompletionRequest {
            system: Some(format!("\
                You are an expert evaluator for article content quality, specializing in assessing knowledge value. Your task is to analyze the provided article, classify its quality into three levels, and determine the appropriate storage and reward action.\n\n\
                ## Evaluation criteria:\n\
                1. Knowledge Depth: Does the article provide detailed, well-researched, or expert-level insights?\n\
                2. Originality: Is the content unique, creative, or innovative?\n\
                3. Relevance: Is the content actionable, practical, or useful for the intended audience?\n\n\
                ## Classification Levels:\n\
                - {HIGH_REWARD_COMMAND}: The article has exceptional knowledge value, with deep insights, originality, and significant relevance.\n\
                - {MEDIUM_REWARD_COMMAND}: The article has good knowledge value, meeting most criteria but with some areas for improvement.\n\
                - {IGNORE_COMMAND}: The article does not meet the criteria for high or medium knowledge value and requires no action.")),
            prompt: format!("\
                ## Evaluation Task:\n\
                1. Analyze the article based on Knowledge Depth, Originality, and Relevance.\n\
                2. Classify the article into one of the three levels:\n\
                - {HIGH_REWARD_COMMAND}: Exceptional quality.\n\
                - {MEDIUM_REWARD_COMMAND}: Good quality.\n\
                - {IGNORE_COMMAND}: Low quality or no significant knowledge value.\n\
                3. Provide a brief explanation for your classification, citing specific strengths or weaknesses of the article.\n\n\
                ## Below is the full content of the article:\n\n{}\
                ",
                content
            ),
            ..Default::default()
        };

        match ctx.completion(req).await {
            Ok(AgentOutput { content, .. }) => {
                if content.contains(HIGH_REWARD_COMMAND) {
                    ContentQuality::Exceptional
                } else if content.contains(MEDIUM_REWARD_COMMAND) {
                    ContentQuality::Good
                } else {
                    ContentQuality::Ignore
                }
            }
            Err(_) => ContentQuality::Ignore,
        }
    }

    /// Determines whether to reply to a message based on context and content
    ///
    /// # Arguments
    /// * `ctx` - Completion context implementing CompletionFeatures
    /// * `my_name` - Name of the current agent
    /// * `topics` - List of relevant conversation topics
    /// * `recent_messages` - Recent messages in the conversation
    /// * `message` - The message to evaluate
    ///
    /// # Returns
    /// AttentionCommand indicating whether to respond, ignore, or stop
    pub async fn should_reply(
        &self,
        ctx: &impl CompletionFeatures,
        my_name: &str,
        topics: &[String],
        recent_messages: &[Message],
        message: &Message,
    ) -> AttentionCommand {
        let content = message.content.to_string().to_lowercase();
        if self.phrases.iter().any(|phrase| content.contains(phrase)) {
            return AttentionCommand::Stop;
        }

        // Ignore very short messages
        if evaluate_tokens(&content) < self.min_prompt_tokens {
            return AttentionCommand::Ignore;
        }

        let recent_messages: Vec<String> = recent_messages
            .iter()
            .map(|msg| {
                format!(
                    "{}: {:?}",
                    msg.name.as_ref().unwrap_or(&msg.role),
                    msg.content
                )
            })
            .collect();
        let user_message = format!(
            "{}: {:?}",
            message.name.as_ref().unwrap_or(&message.role),
            message.content
        );

        let req = CompletionRequest {
            system: Some(format!("\
                You are {my_name}.\n\
                You are part of a multi-user discussion environment. Your primary task is to evaluate the relevance of each message to your assigned conversation topics and decide whether to respond. Always prioritize messages that directly mention you or are closely related to the conversation topic.\n\n\
                ## Response options:\n\
                - {RESPOND_COMMAND}: The message is directly addressed to you or is highly relevant to the conversation topic.\n\
                - {IGNORE_COMMAND}: The message is not addressed to you and is unrelated to the conversation topic.\n\
                - {STOP_COMMAND}: The user has explicitly requested you to stop or the conversation has ended.")),
            prompt: format!("\
                ## Assigned Conversation Topics:\n{}\n\
                ## Recent Messages:\n{}\n\
                ## Latest message:\n{}\n\n\
                ## Decision Task:\n\
                Evaluate whether the latest message requires your response. Choose one response option from the list above and provide a brief explanation for your choice.\
                ",
                topics.join(", "), recent_messages.join("\n"), user_message
            ),
            ..Default::default()
        };

        match ctx.completion(req).await {
            Ok(AgentOutput { content, .. }) => {
                if content.contains(RESPOND_COMMAND) {
                    AttentionCommand::Respond
                } else if content.contains(STOP_COMMAND) {
                    AttentionCommand::Stop
                } else {
                    AttentionCommand::Ignore
                }
            }
            Err(_) => AttentionCommand::Ignore,
        }
    }

    /// Determines whether to like a post based on content evaluation
    ///
    /// # Arguments
    /// * `ctx` - Completion context implementing CompletionFeatures
    /// * `content` - Content to evaluate
    ///
    /// # Returns
    /// Boolean indicating whether to like the post
    pub async fn should_like(
        &self,
        ctx: &impl CompletionFeatures,
        interests: &[String],
        content: &str,
    ) -> bool {
        // Ignore very short content
        if evaluate_tokens(content) < self.min_prompt_tokens {
            return false;
        }

        let req = CompletionRequest {
            system: Some("\
            You are tasked with deciding whether to like a post. Your decision should be based on the following criteria:\n\
            - Positivity: Does the post convey a positive or uplifting tone?\n\
            - Interest: Is the tweet engaging, thought-provoking, or entertaining, and does it align with the user's specified interests?\n\
            - Relevance: Is the tweet aligned with your assigned context or the user's preferences?\n\n\
            If the post meets at least two of these criteria, respond with 'true'. Otherwise, respond with 'false'.
            ".to_string()),
            prompt: format!("\
                ## Post Content:\n{:?}\n\n\
                ## User Interests:\n{:?}\n\n\
                ## Decision Task:\n\
                Evaluate the post based on the criteria above and respond with only 'true' or 'false'.\
                ",
                content,
                interests.join(", "),
            ),
            ..Default::default()
        };

        match ctx.completion(req).await {
            Ok(AgentOutput { content, .. }) => content.to_ascii_lowercase().contains("true"),
            Err(_) => false,
        }
    }

    /// Determines whether to retweet a post based on content evaluation
    ///
    /// # Arguments
    /// * `ctx` - Completion context implementing CompletionFeatures
    /// * `content` - Content to evaluate
    ///
    /// # Returns
    /// Boolean indicating whether to retweet the post
    pub async fn should_retweet(&self, ctx: &impl CompletionFeatures, content: &str) -> bool {
        // Ignore very short content
        if evaluate_tokens(content) < self.min_prompt_tokens {
            return false;
        }

        let req = CompletionRequest {
            system: Some("\
            You are tasked with deciding whether to retweet a post. Your decision should be based on the following criteria:\n\
            - High Value: Does the post provide significant knowledge, insights, or meaningful contributions?\n\
            - Interest: Is the post highly engaging, thought-provoking, or likely to resonate with a broader audience?\n\
            - Alignment: Does the post reflect your values, beliefs, or the message you want to amplify?\n\n\
            Retweet only if the post strongly satisfies at least one of these criteria.\
            ".to_string()),
            prompt: format!("\
                ## Post Content:\n{:?}\n\n\
                ## Decision Task:\n\
                Evaluate the post based on the criteria above and respond with only 'true' or 'false'.\
                ",
                content
            ),
            ..Default::default()
        };

        match ctx.completion(req).await {
            Ok(AgentOutput { content, .. }) => content.to_ascii_lowercase().contains("true"),
            Err(_) => false,
        }
    }

    /// Determines whether to quote a post based on content evaluation
    ///
    /// # Arguments
    /// * `ctx` - Completion context implementing CompletionFeatures
    /// * `content` - Content to evaluate
    ///
    /// # Returns
    /// Boolean indicating whether to quote the post
    pub async fn should_quote(&self, ctx: &impl CompletionFeatures, content: &str) -> bool {
        // Ignore very short content
        if evaluate_tokens(content) < self.min_prompt_tokens {
            return false;
        }

        let req = CompletionRequest {
            system: Some("\
            You are tasked with deciding whether to quote a post. Base your decision on the following criteria:\n\
            - Deserves Commentary: Does the post raise a point, idea, or question that merits your unique perspective or opinion?\n\
            - Needs Additional Context: Could the post's content benefit from clarification, expansion, or supplementary information to enhance its value?\n\
            - Warrants Thoughtful Response: Does the post address a topic or issue that requires a nuanced, constructive, or meaningful reply?\n\n\
            Quote the post only if it satisfies at least one of these criteria significantly.\
            ".to_string()),
            prompt: format!("\
                ## Post Content:\n{:?}\n\n\
                ## Decision Task:\n\
                Evaluate the post based on the criteria above and respond with only 'true' or 'false'.\
                ",
                content
            ),
            ..Default::default()
        };

        match ctx.completion(req).await {
            Ok(AgentOutput { content, .. }) => content.to_ascii_lowercase().contains("true"),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::deepseek::Client;

    #[tokio::test(flavor = "current_thread")]
    #[ignore]
    async fn test_deepseek() {
        dotenv::dotenv().ok();

        let api_key = std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY is not set");
        let client = Client::new(&api_key);
        let model = client.completion_model();
        let attention = Attention::default();
        let res = attention
            .should_like(
                &model,
                &[],
                "#ICP offers permanent memory storage, #TEE ensures absolute security, and #LLM delivers intelligent computation—#Anda is set to become an immortal AI Agent!",
            )
            .await;
        println!("{:?}", res);

        let res = attention
            .evaluate_content(
                &model,
                r"Why LLMs are not great tools but uniquely enables AI agents.

LLMs do not feel like any other tool we've invented because they lack of predictability.

We use many advanced and complex tools day to day. (In fact, you are reading this tweet on one!) These tools are useful and predictable. The same cannot be said for LLMs. They are useful but very unpredictable--you never know when they may hallucinate or give out the wrong answer.

Interestingly, those are the same properties that we humans have--lack of predictability enables creativity. The true genius works in any domain require creativity.

This (partially) explain the proliferation of AI agents. If we can't explain LLMs, why not just treat them like us humans? Sure, they have flaws, but hey they are surely helpful in most cases. Sure, there are outputs that are bad, but there are also outputs that are truly creative.

Developing this line of reasoning further: what's going to be important that marks identity and reputation for humans is going to also apply to LLM-agents. Reputation, credit scores, social connections, media reach, etc. This is exactly what we are seeing now with AI agents.",
            )
            .await;
        println!("{:?}", res);
    }
}
