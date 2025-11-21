/**
 * Regex to detect agent messages in debate mode.
 * Format: [AgentName]: Message content
 */
export const AGENT_REGEX = /^\[(.*?)]: ([\s\S]*)/;
