import http from "k6/http";
import { check } from "k6";
import { textSummary } from "https://jslib.k6.io/k6-summary/0.0.1/index.js";

export const options = {
  scenarios: {
    constant_load: {
      executor: "constant-vus",
      vus: 10,
      duration: __ENV.DURATION || "60s",
      gracefulStop: "3s",
    },
  },
};

const payload = open("./body.json");

// Generate a random token for this request, this ensures gateways do not abuse the
// repetitive nature of the benchmark too much.
function generateRandomToken() {
  return (
    Math.random().toString(36).substring(2) +
    Math.random().toString(36).substring(2)
  );
}

export default function() {
  const params = {
    headers: {
      "Content-Type": "application/json",
      authorization: `Bearer ${generateRandomToken()}`,
    },
  };

  const response = http.post("http://localhost:4000/graphql", payload, params);

  check(response, {
    "is status 200": (resp) => resp.status === 200,
    "response is correct": (resp) => {
      // Too big to commit
      if (resp.body.length === 7893375) {
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
  const stats = http.get("http://localhost:7100/stats");
  data["subgraph_stats"] = stats.json();
  return {
    stdout: textSummary(data, { indent: " ", enableColors: true }),
    "summary.json": JSON.stringify(data),
  };
}
