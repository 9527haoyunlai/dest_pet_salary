const EXACT_DECIMAL = /^([+-]?)(\d+)(?:\.(\d*))?$/;

function incrementDecimalDigits(value: string): string {
  const digits = value.split("");
  for (let index = digits.length - 1; index >= 0; index -= 1) {
    if (digits[index] !== "9") {
      digits[index] = String.fromCharCode(digits[index].charCodeAt(0) + 1);
      return digits.join("");
    }
    digits[index] = "0";
  }
  return `1${digits.join("")}`;
}

export function formatExactCurrency(exact: string): string {
  const match = EXACT_DECIMAL.exec(exact.trim());
  if (!match) return exact;

  const [, sign, integerPart, rawFraction = ""] = match;
  const integer = integerPart.replace(/^0+(?=\d)/, "");
  const fraction = rawFraction.padEnd(3, "0");
  let combined = `${integer}${fraction.slice(0, 2)}`;

  if (fraction[2] >= "5") combined = incrementDecimalDigits(combined);

  const padded = combined.padStart(3, "0");
  const whole = padded.slice(0, -2).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const cents = padded.slice(-2);
  const prefix = sign === "-" && combined !== "000" ? "-" : "";
  return `${prefix}¥${whole}.${cents}`;
}

export function formatCycleRange(start: string, end: string): string {
  return `${start} → ${end}`;
}
