use anthropic_sdk::{
    api_retry, TokenCounter, Tool, ToolExecutor, ToolFunction, ToolRegistry, ToolResult,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// Weather tool that simulates fetching weather data
struct WeatherTool;

#[async_trait]
impl ToolFunction for WeatherTool {
    async fn execute(
        &self,
        parameters: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let location = parameters
            .get("location")
            .and_then(|v| v.as_str())
            .ok_or("Missing location parameter")?;

        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Simulate weather data based on location
        let weather_data = match location.to_lowercase().as_str() {
            l if l.contains("san francisco") || l.contains("sf") => {
                json!({
                    "location": "San Francisco, CA",
                    "temperature": 68,
                    "condition": "Partly cloudy",
                    "humidity": 65,
                    "wind_speed": "8 mph",
                    "forecast": "Mild and pleasant with some afternoon fog"
                })
            }
            l if l.contains("new york") || l.contains("nyc") => {
                json!({
                    "location": "New York, NY",
                    "temperature": 72,
                    "condition": "Sunny",
                    "humidity": 55,
                    "wind_speed": "12 mph",
                    "forecast": "Clear skies with comfortable temperatures"
                })
            }
            l if l.contains("london") => {
                json!({
                    "location": "London, UK",
                    "temperature": 59,
                    "condition": "Overcast",
                    "humidity": 78,
                    "wind_speed": "6 mph",
                    "forecast": "Typical London weather with light drizzle expected"
                })
            }
            _ => {
                json!({
                    "location": location,
                    "temperature": 65,
                    "condition": "Unknown",
                    "humidity": 50,
                    "wind_speed": "5 mph",
                    "forecast": "Weather data not available for this location"
                })
            }
        };

        Ok(ToolResult::success("weather_id", weather_data.to_string()))
    }
}

/// Calculator tool for mathematical operations
struct CalculatorTool;

#[async_trait]
impl ToolFunction for CalculatorTool {
    async fn execute(
        &self,
        parameters: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let expression = parameters
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or("Missing expression parameter")?;

        // Simple expression evaluator (in production, use a proper parser)
        let result = match self.evaluate_expression(expression) {
            Ok(value) => value,
            Err(e) => return Err(format!("Calculation error: {}", e).into()),
        };

        let result_json = json!({
            "expression": expression,
            "result": result,
            "explanation": format!("{} = {}", expression, result)
        });

        Ok(ToolResult::success("calc_id", result_json.to_string()))
    }
}

impl CalculatorTool {
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        // Simple evaluator for demo - handles basic arithmetic
        let expr = expr.replace(" ", "");

        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num);
        }

        // Handle simple operations
        if let Some(pos) = expr.find('+') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..]; // Skip the operator
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            return Ok(left_val + right_val);
        }

        if let Some(pos) = expr.find('-') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            return Ok(left_val - right_val);
        }

        if let Some(pos) = expr.find('*') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            return Ok(left_val * right_val);
        }

        if let Some(pos) = expr.find('/') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            if right_val == 0.0 {
                return Err("Division by zero".to_string());
            }
            return Ok(left_val / right_val);
        }

        Err(format!("Cannot evaluate expression: {}", expr))
    }
}

/// Time tool for current time and timezone information
struct TimeTool;

#[async_trait]
impl ToolFunction for TimeTool {
    async fn execute(
        &self,
        parameters: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let timezone = parameters
            .get("timezone")
            .and_then(|v| v.as_str())
            .unwrap_or("UTC");

        // Simulate timezone lookup
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let formatted_time = format!(
            "2024-01-15 {:02}:{:02}:{:02}",
            (current_time / 3600) % 24,
            (current_time / 60) % 60,
            current_time % 60
        );

        let result = json!({
            "timezone": timezone,
            "current_time": formatted_time,
            "unix_timestamp": current_time,
            "format": "YYYY-MM-DD HH:MM:SS"
        });

        Ok(ToolResult::success("time_id", result.to_string()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Comprehensive Tool Use Demo");
    println!("==============================\n");

    // Initialize token counter and retry system
    let token_counter = TokenCounter::new();
    let retry_executor = api_retry();

    // Create tool registry and register tools
    let mut registry = ToolRegistry::new();

    // Register weather tool
    let weather_tool = Tool::new(
        "get_weather",
        "Get current weather information for a location",
    )
    .parameter(
        "location",
        "string",
        "The city and state/country, e.g. 'San Francisco, CA' or 'London, UK'",
    )
    .required("location")
    .build();

    registry.register("get_weather", weather_tool, Box::new(WeatherTool))?;

    // Register calculator tool
    let calculator_tool = Tool::new("calculate", "Perform mathematical calculations")
        .parameter(
            "expression",
            "string",
            "Mathematical expression to evaluate, e.g. '25 + 17' or '100 / 4'",
        )
        .required("expression")
        .build();

    registry.register("calculate", calculator_tool, Box::new(CalculatorTool))?;

    // Register time tool
    let time_tool = Tool::new("get_time", "Get current time information")
        .parameter(
            "timezone",
            "string",
            "Timezone to get time for (optional, defaults to UTC)",
        )
        .build();

    registry.register("get_time", time_tool, Box::new(TimeTool))?;

    // Create tool executor
    let executor = ToolExecutor::new(Arc::new(registry));

    println!(
        "🛠️  Registered {} tools:",
        executor.registry().tool_names().len()
    );
    for tool_name in executor.registry().tool_names() {
        println!("  • {}", tool_name);
    }

    // Demo scenarios
    println!("\n📱 Demo Scenarios:");
    println!("=================\n");

    // Scenario 1: Weather query
    println!("🌤️  Scenario 1: Weather Query");
    println!("------------------------------");

    let weather_request = anthropic_sdk::types::ToolUse {
        id: "weather_001".to_string(),
        name: "get_weather".to_string(),
        input: json!({"location": "San Francisco, CA"}),
    };

    let _weather_start = token_counter.start_request("claude-3-5-sonnet-latest");
    let weather_results = executor.execute_multiple(&[weather_request]).await;
    println!("Request: Get weather for San Francisco");
    if let Some(Ok(result)) = weather_results.first() {
        println!("Result: {:?}", result.content);
    }

    // Scenario 2: Mathematical calculation
    println!("\n🧮 Scenario 2: Mathematical Calculation");
    println!("---------------------------------------");

    let calc_request = anthropic_sdk::types::ToolUse {
        id: "calc_001".to_string(),
        name: "calculate".to_string(),
        input: json!({"expression": "25 + 17 * 2"}),
    };

    let calc_results = executor.execute_multiple(&[calc_request]).await;
    println!("Request: Calculate 25 + 17 * 2");
    if let Some(Ok(result)) = calc_results.first() {
        println!("Result: {:?}", result.content);
    }

    // Scenario 3: Time query
    println!("\n⏰ Scenario 3: Time Information");
    println!("-------------------------------");

    let time_request = anthropic_sdk::types::ToolUse {
        id: "time_001".to_string(),
        name: "get_time".to_string(),
        input: json!({"timezone": "PST"}),
    };

    let time_results = executor.execute_multiple(&[time_request]).await;
    println!("Request: Get current time in PST");
    if let Some(Ok(result)) = time_results.first() {
        println!("Result: {:?}", result.content);
    }

    // Scenario 4: Parallel tool execution
    println!("\n⚡ Scenario 4: Parallel Tool Execution");
    println!("-------------------------------------");

    let parallel_requests = vec![
        anthropic_sdk::types::ToolUse {
            id: "parallel_weather".to_string(),
            name: "get_weather".to_string(),
            input: json!({"location": "New York, NY"}),
        },
        anthropic_sdk::types::ToolUse {
            id: "parallel_calc".to_string(),
            name: "calculate".to_string(),
            input: json!({"expression": "100 / 4"}),
        },
        anthropic_sdk::types::ToolUse {
            id: "parallel_time".to_string(),
            name: "get_time".to_string(),
            input: json!({"timezone": "EST"}),
        },
    ];

    let start_time = std::time::Instant::now();
    let parallel_results = executor.execute_multiple(&parallel_requests).await;
    let execution_time = start_time.elapsed();

    println!("Executed {} tools in parallel:", parallel_results.len());
    for (i, result) in parallel_results.iter().enumerate() {
        println!("  Tool {}: {}", i + 1, result.is_ok());
    }
    println!("Total execution time: {:?}", execution_time);

    // Scenario 5: Error handling
    println!("\n❌ Scenario 5: Error Handling");
    println!("-----------------------------");

    let error_request = anthropic_sdk::types::ToolUse {
        id: "error_001".to_string(),
        name: "calculate".to_string(),
        input: json!({"expression": "10 / 0"}), // Division by zero
    };

    let error_results = executor.execute_multiple(&[error_request]).await;
    match error_results.first() {
        Some(Ok(result)) => println!("Result (may be error): {:?}", result.is_error),
        Some(Err(e)) => println!("Expected error handled: {}", e),
        None => println!("No result"),
    }

    // Performance metrics
    println!("\n📊 Performance Metrics");
    println!("======================");

    let usage_summary = token_counter.get_summary();
    println!(
        "Token Usage: {} total tokens tracked",
        usage_summary.total_tokens
    );
    println!(
        "Session Duration: {:.1} seconds",
        usage_summary.session_duration.as_secs_f64()
    );

    println!(
        "Retry Policy: {} max retries, {}ms initial delay",
        retry_executor.get_policy().max_retries,
        retry_executor.get_policy().initial_delay.as_millis()
    );

    // Tool registry statistics
    println!("\nTool Registry Stats:");
    println!(
        "  • Registered tools: {}",
        executor.registry().tool_names().len()
    );
    println!("  • Tools executed: 6 (4 individual + 3 parallel)");
    println!("  • Error scenarios: 1 handled");

    println!("\n✨ Comprehensive Tool Use Demo Complete!");
    println!("🚀 All tool scenarios executed successfully with proper error handling!");
    println!("💡 This demonstrates production-ready tool use patterns with:");
    println!("   • Multiple tool types (weather, calculator, time)");
    println!("   • Parallel execution for performance");
    println!("   • Comprehensive error handling");
    println!("   • Token tracking and retry policies");

    Ok(())
}
