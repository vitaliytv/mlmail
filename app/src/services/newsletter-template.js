import { invoke } from '@tauri-apps/api/core'

/**
 * @typedef {{ id: string, name: string, from_pattern: string, prompt: string }} NewsletterTemplate
 * @typedef {{ title: string, url: string, description: string }} NewsletterArticle
 */

export async function listTemplates() {
  return /** @type {NewsletterTemplate[]} */ (await invoke('newsletter_template_list'))
}

export async function saveTemplate(template) {
  await invoke('newsletter_template_save', { template })
}

export async function deleteTemplate(id) {
  await invoke('newsletter_template_delete', { id })
}

/** Save template as a bundled system template (dev-only). */
export async function saveBuiltinTemplate(template) {
  await invoke('newsletter_template_save_builtin', { template })
}

/**
 * Find the first template that matches the message's from and/or subject.
 * A non-empty pattern field must match (case-insensitive substring).
 * Both non-empty fields must match simultaneously.
 * @param {{ from?: string, subject?: string }} message
 * @param {NewsletterTemplate[]} templates
 * @returns {NewsletterTemplate | null}
 */
export function findTemplateForMessage(message, templates) {
  const lowerFrom = (message?.from ?? '').toLowerCase()
  const lowerSubject = (message?.subject ?? '').toLowerCase()
  return templates.find(t => {
    const fromOk = !t.from_pattern || lowerFrom.includes(t.from_pattern.toLowerCase())
    const subjectOk = !t.subject_pattern || lowerSubject.includes(t.subject_pattern.toLowerCase())
    return fromOk && subjectOk && (t.from_pattern || t.subject_pattern)
  }) ?? null
}

/** Generate a slug id from a sender email or domain string. */
export function slugify(str) {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
}

export const SYSTEM_PROMPT = [
  'You are an assistant that extracts articles from newsletter emails.',
  'Given an email body and user-defined extraction instructions, return ONLY a valid JSON array.',
  'Each element: {"title": "...", "url": "https://...", "description": "..."}',
  'Translate title and description to Ukrainian.',
  'The url must be a real link present in the email — never invent URLs.',
  'If no articles found, return [].',
  'Return nothing except the JSON array — no markdown, no explanation.',
].join(' ')
