/**
 * Short, calm status verbs displayed while waiting for the model to respond.
 */
const SPINNER_VERBS = [
  "Calculating",
  "Computing",
  "Considering",
  "Contemplating",
  "Deciphering",
  "Deliberating",
  "Determining",
  "Generating",
  "Inferring",
  "Perusing",
  "Processing",
  "Proofing",
  "Synthesizing",
  "Thinking",
  "Working",
];

/** Pick a random spinner verb. */
export function randomSpinnerVerb(): string {
  return SPINNER_VERBS[Math.floor(Math.random() * SPINNER_VERBS.length)];
}
