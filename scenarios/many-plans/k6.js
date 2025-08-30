import http from "k6/http";
import { check } from "k6";
import { textSummary } from "https://jslib.k6.io/k6-summary/0.1.0/index.js";

export const options = {
  scenarios: {
    open_model: {
      executor: "constant-vus",
      vus: 1,
      duration: __ENV.DURATION || "30s",
    },
  },
};

const payload = open("./body.json");
const expected = open("./expected.json");
const params = {
  headers: {
    "Content-Type": "application/json",
  },
};

export default function () {
  const res = http.post("http://localhost:4000/graphql", payload, params);

  check(res, {
    "response code was 200": (res) => res.status === 200,
    "response is correct": (resp) => {
      if (resp.body === expected) {
        return true;
      }

      console.log("Incorrect response", `Size:`, resp.body.length);

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
          resp.body,
        );
      }

      return false;
    },
  });
}

export function handleSummary(data) {
  const stats = http.get("http://localhost:7000/stats");
  data["subgraph_stats"] = stats.json();
  return {
    "summary.json": JSON.stringify(data),
    stdout: textSummary(data, { indent: " ", enableColors: true }),
  };
}
