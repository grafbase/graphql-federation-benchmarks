//! Copied and adjust from The Guild's GraphQL Gateways Benchmark
//! https://github.com/graphql-hive/graphql-gateways-benchmark
import http from "k6/http";
import { check } from "k6";
import { textSummary } from "https://jslib.k6.io/k6-summary/0.1.0/index.js";

export const options = {
  scenarios: {
    constant_rate: {
      executor: "constant-arrival-rate",
      rate: 500,
      timeUnit: "1s",
      duration: __ENV.DURATION || "60s",
      preAllocatedVUs: 10,
      maxVUs: 200,
      gracefulStop: "3s",
    },
  },
};

const payload = JSON.stringify({
  query: `
    fragment User on User {
      id
      username
      name
    }

    fragment Review on Review {
      id
      body
    }

    fragment Product on Product {
      inStock
      name
      price
      shippingEstimate
      upc
      weight
    }

    query TestQuery {
      users {
        ...User
        reviews {
          ...Review
          product {
            ...Product
            reviews {
              ...Review
              author {
                ...User
                reviews {
                  ...Review
                  product {
                    ...Product
                  }
                }
              }
            }
          }
        }
      }
      topProducts {
        ...Product
        reviews {
          ...Review
          author {
            ...User
            reviews {
              ...Review
              product {
                ...Product
              }
            }
          }
        }
      }
    }`,
});
const expected = open("./expected.json");

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
    "response code was 200": (resp) => resp.status === 200,
    "response is correct": (resp) => {
      // Hive doesn't return the fields with the expected ordering, so only comparing length.
      if (resp.body.length === expected.length) {
        return true;
      }

      console.log("Incorrect response", `Size:`, resp.body.length);

      if (resp.body.length < 1000) {
        console.log("Response:", resp.body);
      } else {
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
      }

      return false;
    },
  });
}

export function handleSummary(data) {
  const stats = http.get("http://localhost:7200/stats");
  data["subgraph_stats"] = stats.json();
  return {
    "summary.json": JSON.stringify(data),
    stdout: textSummary(data, { indent: " ", enableColors: true }),
  };
}
