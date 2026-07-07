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

const SPINNER_VERBS_JA = [
  "確認中",
  "計算中",
  "検討中",
  "解釈中",
  "生成中",
  "推論中",
  "処理中",
  "校正中",
  "整理中",
  "思考中",
  "作業中",
];

/** Pick a random spinner verb. */
export function randomSpinnerVerb(locale = "en"): string {
  const verbs = locale === "ja" ? SPINNER_VERBS_JA : SPINNER_VERBS;
  return verbs[Math.floor(Math.random() * verbs.length)];
}
