import http from "k6/http";
import { check } from "k6";
import { textSummary } from "https://jslib.k6.io/k6-summary/0.1.0/index.js";

export const options = {
  scenarios: {
    open_model: {
      executor: "constant-vus",
      vus: 1,
      duration: "5s",
    },
  },
};

const payload = open("./request/body.json");
const expected = open("./request/expected.json");
const params = {
  headers: {
    "Content-Type": "application/json",
  },
};

export default function() {
  const res = http.post("http://localhost:4000", payload, params);

  check(res, {
    "response code was 200": (res) => res.status == 200,
    "no graphql errors": (resp) => {
      const json = resp.json();
      const noErrors =
        !!json &&
        typeof json === "object" &&
        !Array.isArray(json) &&
        !json.errors;

      if (!noErrors) {
        console.log(
          "graphql_errors",
          `‼️ Got GraphQL errors, here's a sample:`,
          res.body,
        );
      }

      return noErrors;
    },
    "response matches expected": (res) => {
      return res.body === expected;
    },
  });
}

export function handleSummary(data) {
  return {
    "summary.json": JSON.stringify(data),
    stdout: textSummary(data, { indent: " ", enableColors: true }),
  };
}
