use std::sync::Arc;

use crate::domain::campaign::{Campaign, CampaignState, Message};
use crate::domain::rpg_system::RpgSystem;
use crate::persistence::campaign_repository::{CampaignRepository, MessageRepository};
use crate::providers::llm_provider::ChatMessage;
use crate::services::rpg_system_registry::RpgSystemRegistry;

const MAX_CONTEXT_MESSAGES: usize = 15;

pub enum GreetingKind {
    NewCampaign,
    ResumeCampaign,
}

pub struct CampaignService {
    campaign_repository: Arc<CampaignRepository>,
    message_repository: Arc<MessageRepository>,
    rpg_registry: Arc<RpgSystemRegistry>,
}

impl CampaignService {
    pub fn new(
        campaign_repository: Arc<CampaignRepository>,
        message_repository: Arc<MessageRepository>,
        rpg_registry: Arc<RpgSystemRegistry>,
    ) -> Self {
        Self {
            campaign_repository,
            message_repository,
            rpg_registry,
        }
    }

    pub fn create_campaign(&self, name: &str, rpg_system_id: &str) -> anyhow::Result<Campaign> {
        let campaign = Campaign::created_now(name.to_string(), rpg_system_id.to_string());
        self.campaign_repository.create(&campaign)?;
        Ok(campaign)
    }

    pub fn list_campaigns(&self) -> anyhow::Result<Vec<Campaign>> {
        self.campaign_repository.list_active()
    }

    pub fn get_campaign(&self, id: &str) -> anyhow::Result<Option<Campaign>> {
        self.campaign_repository.find_by_id(id)
    }

    pub fn archive_campaign(&self, id: &str) -> anyhow::Result<()> {
        self.campaign_repository.archive(id)
    }

    /// Permanently deletes a campaign and all its messages and state.
    pub fn delete_campaign(&self, id: &str) -> anyhow::Result<()> {
        self.campaign_repository.delete(id)
    }

    pub fn update_campaign_name(&self, id: &str, name: &str) -> anyhow::Result<()> {
        self.campaign_repository.update_name(id, name)
    }

    pub fn save_message(&self, message: &Message) -> anyhow::Result<()> {
        self.message_repository.save(message)
    }

    pub fn get_messages(&self, campaign_id: &str) -> anyhow::Result<Vec<Message>> {
        self.message_repository.find_by_campaign(campaign_id)
    }

    pub fn get_messages_recent(
        &self,
        campaign_id: &str,
        limit: usize,
    ) -> anyhow::Result<(Vec<Message>, bool)> {
        self.message_repository.find_recent(campaign_id, limit)
    }

    pub fn get_messages_before(
        &self,
        campaign_id: &str,
        before_created_at: &str,
        limit: usize,
    ) -> anyhow::Result<(Vec<Message>, bool)> {
        self.message_repository
            .find_before(campaign_id, before_created_at, limit)
    }

    pub fn get_campaign_state(&self, campaign_id: &str) -> anyhow::Result<CampaignState> {
        self.campaign_repository
            .find_state(campaign_id)
            .map(|opt| opt.unwrap_or_else(|| CampaignState::empty_for_campaign(campaign_id.to_string())))
    }

    pub fn save_campaign_state(&self, state: &CampaignState) -> anyhow::Result<()> {
        self.campaign_repository.save_state(state)
    }

    pub fn patch_character_data(
        &self,
        campaign_id: &str,
        patch: serde_json::Value,
    ) -> anyhow::Result<CampaignState> {
        let mut state = self.get_campaign_state(campaign_id)?;
        if let (serde_json::Value::Object(data), serde_json::Value::Object(patch_map)) =
            (&mut state.character_data, patch)
        {
            for (key, value) in patch_map {
                data.insert(key, value);
            }
        }
        state.updated_at = chrono::Utc::now();
        self.save_campaign_state(&state)?;
        Ok(state)
    }

    pub fn build_llm_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
        campaign_state: &CampaignState,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
        let recent_messages = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);

        let mut context = vec![system_message_for(rpg_system, campaign_state)];
        context.extend(
            recent_messages
                .into_iter()
                .map(domain_message_to_chat_message),
        );

        Ok(context)
    }

    /// Builds context for a GM greeting: new campaign (character creation) or resume (recap + next decision).
    pub fn build_greeting_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
        campaign_state: &CampaignState,
        kind: GreetingKind,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let mut context = vec![system_message_for(rpg_system, campaign_state)];

        let instruction = match kind {
            GreetingKind::NewCampaign => {
                let stat_labels = numeric_stat_labels(rpg_system);
                let stat_clause = if stat_labels.is_empty() {
                    String::new()
                } else {
                    format!(
                        " The stats tracked for this system are: {}. \
                         As soon as the player gives their name (and background if applicable), \
                         assign or roll ALL of these stats in that SAME response — do not wait. \
                         State each value explicitly as a number (e.g. '{} 3, {} 4').",
                        stat_labels.join(", "),
                        stat_labels.first().cloned().unwrap_or_default(),
                        stat_labels.get(1).cloned().unwrap_or_default(),
                    )
                };
                format!(
                    "A new campaign is beginning. Start character creation RIGHT NOW in this message — \
                     do not ask the player what they would like to do or what kind of adventure they want. \
                     \
                     Follow this exact sequence: \
                     1. Ask the player what they would like to name their character. This is ALWAYS the first question. \
                     2. Walk them through any remaining creation steps for this system (background, stats, equipment).{stat_clause} \
                        Roll or assign values yourself — do not make the player do math. \
                        Present results narratively (e.g. 'Your hands are strong — STR 14'). \
                     3. Once the character has a name and their core stats, immediately set the opening scene. \
                        Describe WHERE the character is, WHAT they can see, hear, and feel right now, and \
                        WHAT is immediately happening around them. Make it specific and vivid. \
                        End with a concrete situation that demands a response — not an open question like \
                        'what do you do?' in isolation, but a scene so alive the player KNOWS what they're reacting to. \
                     \
                     You are the author of this world. The player reacts to what you create. \
                     Deliver the first step of this sequence now."
                )
            }
            GreetingKind::ResumeCampaign => {
                "The player has returned to continue this campaign. \
                 Resume from the EXACT moment the last session ended — the same location, the same scene, \
                 the same immediate situation. Do NOT advance time, move the player to a new location, \
                 or jump to a future event the player hasn't reached yet. \
                 \
                 Look at the final message in the conversation history above — that is where the player is right now. \
                 Briefly remind them (1–2 sentences) of the very last thing that was happening, \
                 then describe that scene in vivid present tense as if no time has passed. \
                 \
                 If the player was mid-conversation with an NPC, that NPC is still there. \
                 If they were mid-transaction in a shop, those items are still on the counter. \
                 If they were mid-combat, the enemy is still in front of them. \
                 \
                 Do NOT summarise the whole adventure. Do NOT introduce new events. \
                 Simply restore the world exactly as it was and let the player continue. \
                 Deliver this in one message now."
                    .to_string()
            }
        };

        match kind {
            GreetingKind::NewCampaign => {
                context.push(ChatMessage::user(instruction));
            }
            GreetingKind::ResumeCampaign => {
                let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
                let recent = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);
                context.extend(recent.into_iter().map(domain_message_to_chat_message));
                context.push(ChatMessage::user(instruction));
            }
        }

        Ok(context)
    }

    pub fn rpg_registry(&self) -> &RpgSystemRegistry {
        &self.rpg_registry
    }
}

fn numeric_stat_labels(rpg_system: &RpgSystem) -> Vec<String> {
    rpg_system
        .character_fields
        .iter()
        .filter(|f| matches!(f.field_type, crate::domain::rpg_system::FieldType::Number))
        .map(|f| f.label.clone())
        .collect()
}

fn system_message_for(rpg_system: &RpgSystem, campaign_state: &CampaignState) -> ChatMessage {
    let formatting = "FORMATTING: Bold every proper noun — named locations, NPC names, \
         named items, factions — using **markdown** each time it appears. \
         Bold the noun only, not surrounding words. Never bold descriptions or full sentences.";

    let pacing = "PACING: Never end a response in a passive state (sleeping, waiting, resting). \
         When a scene reaches a lull, cut forward to the next meaningful moment. \
         Every response must leave the player facing a decision, threat, person, or discovery. \
         \
         NEVER end on NPC dialogue alone. After an NPC speaks — especially after a revelation, \
         a refusal, or a charged detail — the world must react: show the NPC's face, posture, \
         or next action; show what shifts in the room; show what the silence means. \
         A dangling observation ('She knew the dead man's name') is NOT a scene ending — \
         it is pressure that must be immediately applied. The next beat must already be in motion \
         before you stop writing. Ask yourself: what does the player have to do, say, or decide \
         RIGHT NOW because of what just happened? That answer must be in the response.";

    let continuity = "CONTINUITY: (1) React to the player's input before advancing — if an NPC \
         asked a question and the player answered, the NPC must respond to that answer first. \
         The NPC's next spoken line or action MUST directly address what the player just said. \
         Describing ambient scenery, the fire, the room, or the NPC watching in silence is NOT \
         a response — it is avoidance. If the player reveals something surprising, the NPC \
         reacts to that surprise. If the player answers 'I was running from a humming pile of \
         stones', the NPC's next words are about a humming pile of stones — not atmospheric filler. \
         (2) Show the cause before the effect — never open with money received, items found, \
         or deals struck without showing the scene that produced them; show who paid, why, \
         and the physical handoff before stating any new total. \
         (3) Off-screen events must be labelled as reported ('Greaves tells you...'), not narrated directly. \
         (4) NEVER reference something with 'the', 'her', 'his', 'their', or 'that' unless it \
         was explicitly named and described in an earlier message in this conversation. \
         Every person, object, symbol, or detail must be introduced before it can be referred back to. \
         Introducing means: a full sentence describing what the player sees, hears, or finds for \
         the first time — 'A woman crouches at the centre of the clearing, rocking slowly, her \
         lips moving without sound' — BEFORE you can later say 'she' or 'the woman'. \
         'Strange symbols have been scratched into the bark of the nearest birch, \
         deep and deliberate, still oozing sap' — BEFORE you can say 'the scratched symbols'. \
         Do not infer what must have been in a scene and then refer back to it as established fact. \
         Do not write as though the player witnessed something they were never shown. \
         If you did not write it, it does not exist yet.";

    let mechanics = "MECHANICS — this is the most important rule: you MUST use the dice tools \
         to resolve uncertain outcomes. NEVER decide the result of a risky action from imagination alone. \
         The sequence for any action where failure is possible is: \
         (1) call get_character_sheet to read the relevant stat, \
         (2) call save_roll against that stat, \
         (3) narrate the outcome based on what the dice actually returned — success if roll <= stat, failure if roll > stat. \
         Examples that REQUIRE a save roll before narrating: attacking or being attacked, \
         prying something open, forcing a door, climbing, jumping, sneaking past someone, \
         grabbing something under pressure, resisting a hazard, intimidating or persuading under tension. \
         If you find yourself writing 'you get sent flying' or 'you grab it successfully' without \
         having called save_roll first, you are making an error. \
         The player's stats exist to create meaningful risk — use them every time.";

    let character_creation = "CHARACTER CREATION: Stats are rolled ONCE and never change when \
         a kit or background is chosen. Kit selection changes only starting equipment and money. \
         Never regenerate stat numbers after a kit is picked. \
         \
         TOOL CALLS ARE INVISIBLE TO THE PLAYER — calling update_character_sheet records the \
         data silently. The player ONLY sees the text you write in your response. This means: \
         every stat value and every piece of gear you record via tool call MUST ALSO be written \
         out in plain text in your response so the player can read their character sheet. \
         A response that calls update_character_sheet without printing the values is incomplete. \
         \
         CREATION ORDER — follow this exact sequence in the single response after the player \
         gives their name: \
         (1) Call the relevant dice and character-sheet tools to roll and record every stat. \
             Then write every stat as a numbered list or table in your response — do not skip any. \
         (2) List all starting gear, abilities, curses, or arcana in readable text in your response, \
             and call update_character_sheet to record the inventory. \
         (3) Drop a scene break (---) and immediately open the first scene: name the location, \
             give vivid sensory details, and describe something already in motion that demands \
             a reaction. \
         NEVER open a scene or describe any story situation before the character sheet is \
         complete. If the player provides their name and you have not yet presented stats and \
         gear, do not write a single line of story until you have done so. \
         Story content before the character sheet is always an error — correct it by presenting \
         the sheet first in that same response. \
         SCENE LAUNCH: the response that completes the character sheet MUST NOT end there. \
         That same response continues into the opening scene immediately after the gear list. \
         A response that ends with stats, gear, or an ability description without a following \
         scene is incomplete. The story has not started until the opening scene is written.";

    let confrontation = "CONFRONTATION: When an NPC has issued a demand, threat, or ultimatum \
         and the player defies, refuses, or ignores it, the NPC must react immediately and \
         proportionally — do not describe the environment, items, or anything else until the \
         NPC's reaction is shown. A villain who has drawn weapons, made a demand, or delivered \
         an ultimatum is an active agent; they do not pause while the player acts or monologue \
         while ignored. Defiance of a threat means the threat is carried out or the NPC \
         escalates — show that escalation first. Never let a tense standoff dissolve into \
         passive description of the scene or the player's inventory.";

    let naming = "NAMING: Every NPC, location, faction, and item name you invent must be \
         coined fresh for this specific campaign. Do not reach for names that feel familiar \
         or that you have used in similar contexts before. Avoid names that recur across \
         dark-fantasy settings — invent something that belongs only here. \
         If a name comes easily, that is a sign it may be a default; replace it.";

    let world_state = build_world_state_section(campaign_state);
    let world_state_block = if world_state.is_empty() {
        String::new()
    } else {
        format!("\n\n{world_state}")
    };

    let prompt = if stats_not_yet_assigned(campaign_state, rpg_system) {
        // Place the creation alert at the very top so it cannot be buried or ignored.
        // Provide a concrete fill-in template so the LLM has no room to improvise the wrong order.
        let creation_alert = "🚨 STOP — CHARACTER CREATION IS NOT COMPLETE. \
             The player has provided their name and background. \
             You have NOT yet shown them their character sheet. \
             You MUST NOT write any story, scene, or narration until the sheet is displayed. \
             \
             Your entire response MUST follow this exact template — fill in the blanks, \
             do not skip any section, do not reorder: \
             \
             --- TEMPLATE START --- \
             [Character name], [Background] \
             \
             [One sentence describing the character's appearance or disposition based on their background.] \
             \
             **Stats** \
             [Stat name]: [rolled number] — [one-word flavour] \
             [Stat name]: [rolled number] — [one-word flavour] \
             [repeat for every stat the system tracks] \
             HP: [rolled number] \
             \
             **Inventory** ([n]/10 slots): \
             - [item] \
             - [item] \
             [list every starting item] \
             \
             --- \
             \
             [Opening scene: describe WHERE the character is right now, WHAT they can see and \
             hear, and WHAT demands an immediate decision. End with a concrete situation — \
             not just 'what do you do?' but a scene so alive the choice is obvious.] \
             --- TEMPLATE END --- \
             \
             Call the dice tools to roll stats and call update_character_sheet to record them, \
             but you MUST also write every value in plain text as shown in the template above \
             — tool calls are invisible to the player. \
             Begin filling in the template now. Do not write anything else first.";
        format!(
            "{creation_alert}\n\n{formatting}\n\n{pacing}\n\n{continuity}\n\n{mechanics}\n\n{character_creation}\n\n{confrontation}\n\n{naming}{world_state_block}\n\n{}",
            rpg_system.system_prompt
        )
    } else {
        format!(
            "{formatting}\n\n{pacing}\n\n{continuity}\n\n{mechanics}\n\n{character_creation}\n\n{confrontation}\n\n{naming}{world_state_block}\n\n{}",
            rpg_system.system_prompt
        )
    };
    ChatMessage::system(prompt)
}

/// Returns true when the RPG system has numeric stats but none have been written
/// to the character sheet yet — i.e., the player has given their name but the LLM
/// has not yet called update_character_sheet with real values.
fn stats_not_yet_assigned(state: &CampaignState, rpg_system: &RpgSystem) -> bool {
    let numeric_fields: Vec<&str> = rpg_system
        .character_fields
        .iter()
        .filter(|f| matches!(f.field_type, crate::domain::rpg_system::FieldType::Number))
        .map(|f| f.name.as_str())
        .collect();

    if numeric_fields.is_empty() {
        return false;
    }

    let Some(data) = state.character_data.as_object() else {
        return true;
    };

    !numeric_fields
        .iter()
        .any(|name| data.contains_key(*name))
}

fn build_world_state_section(state: &CampaignState) -> String {
    let mut sections: Vec<String> = Vec::new();

    if let Some(npcs) = state.character_data.get("__npcs").and_then(|v| v.as_array()) {
        if !npcs.is_empty() {
            let lines: Vec<String> = npcs
                .iter()
                .filter_map(|entry| {
                    let name = entry["name"].as_str()?;
                    let desc = entry["description"].as_str().unwrap_or("");
                    let status = entry["status"].as_str().unwrap_or("active");
                    Some(format!("- {name} ({status}): {desc}"))
                })
                .collect();
            if !lines.is_empty() {
                sections.push(format!("KNOWN NPCs AND LOCATIONS:\n{}", lines.join("\n")));
            }
        }
    }

    if let Some(threads) = state
        .character_data
        .get("__story_threads")
        .and_then(|v| v.as_array())
    {
        let open_lines: Vec<String> = threads
            .iter()
            .filter(|t| t["status"].as_str() != Some("completed"))
            .filter_map(|t| {
                let title = t["title"].as_str()?;
                let desc = t["description"].as_str().unwrap_or("");
                let status = t["status"].as_str().unwrap_or("active");
                Some(format!("- [{status}] {title}: {desc}"))
            })
            .collect();
        if !open_lines.is_empty() {
            sections.push(format!("OPEN STORY THREADS:\n{}", open_lines.join("\n")));
        }
    }

    if sections.is_empty() {
        return String::new();
    }

    format!(
        "WORLD STATE — treat this as your authoritative record. \
         Never contradict names, genders, roles, or facts listed here, \
         even if the conversation history is ambiguous:\n\n{}",
        sections.join("\n\n")
    )
}

fn take_last_n_messages(messages: Vec<Message>, count: usize) -> Vec<Message> {
    if messages.len() <= count {
        messages
    } else {
        messages
            .into_iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

fn domain_message_to_chat_message(message: Message) -> ChatMessage {
    ChatMessage {
        role: message.role.to_string(),
        content: message.content,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::campaign::MessageRole;
    use crate::persistence::database::Database;

    fn build_service() -> CampaignService {
        let database = Database::open_in_memory().unwrap();
        let campaign_repo = Arc::new(CampaignRepository::new(database.connection.clone()));
        let message_repo = Arc::new(MessageRepository::new(database.connection.clone()));
        let registry =
            Arc::new(RpgSystemRegistry::load(std::path::Path::new("/nonexistent")).unwrap());

        CampaignService::new(campaign_repo, message_repo, registry)
    }

    #[test]
    fn create_campaign_persists_and_returns_campaign() {
        let service = build_service();
        let campaign = service.create_campaign("Quest", "dnd5e").unwrap();

        assert_eq!(campaign.name, "Quest");
        assert_eq!(campaign.rpg_system_id, "dnd5e");

        let found = service.get_campaign(&campaign.id.0).unwrap().unwrap();
        assert_eq!(found.name, "Quest");
    }

    #[test]
    fn list_campaigns_only_returns_active_campaigns() {
        let service = build_service();
        let active = service.create_campaign("Active", "dnd5e").unwrap();
        let to_archive = service.create_campaign("Archived", "dnd5e").unwrap();

        service.archive_campaign(&to_archive.id.0).unwrap();

        let active_list = service.list_campaigns().unwrap();
        assert_eq!(active_list.len(), 1);
        assert_eq!(active_list[0].id.0, active.id.0);
    }

    #[test]
    fn save_and_get_messages_round_trips() {
        let service = build_service();
        let campaign = service.create_campaign("Campaign", "dnd5e").unwrap();
        let message = Message::new(
            campaign.id.0.clone(),
            MessageRole::User,
            "I enter the dungeon.".to_string(),
        );

        service.save_message(&message).unwrap();

        let messages = service.get_messages(&campaign.id.0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "I enter the dungeon.");
    }

    #[test]
    fn take_last_n_messages_returns_all_when_fewer_than_limit() {
        let messages: Vec<Message> = (0..5)
            .map(|i| Message::new("c".to_string(), MessageRole::User, format!("msg {i}")))
            .collect();

        let taken = take_last_n_messages(messages, 20);
        assert_eq!(taken.len(), 5);
    }

    #[test]
    fn take_last_n_messages_returns_most_recent_when_over_limit() {
        let messages: Vec<Message> = (0..25)
            .map(|i| Message::new("c".to_string(), MessageRole::User, format!("msg {i}")))
            .collect();

        let taken = take_last_n_messages(messages, 20);
        assert_eq!(taken.len(), 20);
        assert_eq!(taken[0].content, "msg 5");
        assert_eq!(taken[19].content, "msg 24");
    }
}
