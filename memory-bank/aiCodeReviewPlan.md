# AI Code Review Integration Plan: GooseBot

## Overview
This document outlines the implementation plan for integrating AI-driven code reviews into Goose's development process. GooseBot, powered by a Large Language Model (LLM), will provide automated, consistent feedback on pull requests to complement human reviewers.

## Purpose and Value
GooseBot will enhance Goose's development process by:

- Providing quick, consistent feedback on PRs
- Complementing (not replacing) human reviewers
- Evaluating code changes across multiple dimensions:
  - Purpose & Documentation Clarity
  - Code Correctness & Rust Best Practices
  - Performance Implications
  - Security Implications
  - Style & Consistency
  - Project Goals Alignment

## Implementation Timeline

### Phase 1: Basic PR Clarity Review
**Goal**: Establish a minimal AI review focused on PR clarity and documentation.

**Implementation Steps:**
1. Create a GitHub Actions workflow triggered on PR events (opened, updated)
   ```yaml
   name: GooseBot Review (Phase 1)
   on:
     pull_request:
       types: [opened, synchronize, reopened]
   ```
2. Set up a job that invokes the LLM via API
   ```yaml
   jobs:
     ai-review:
       runs-on: ubuntu-latest
       permissions:
         pull-requests: write
         contents: read
       steps:
         - uses: actions/checkout@v3
           with:
             fetch-depth: 0  # Get full history for better context
         - name: GooseBot Review - PR Clarity
           env:
             ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
             ANTHROPIC_API_URL: ${{ secrets.ANTHROPIC_API_URL }}  # For internal hosted version
             GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
             PR_REVIEW_WHITELIST: "*.rs,*.md"
             PR_REVIEW_BLACKLIST: "tests/*,benches/*"
           run: |
             # Call script to analyze PR and post comment
             python scripts/goosebot_review.py --scope "clarity" --pr ${{ github.event.pull_request.number }}
   ```
3. Create a script that:
   - Fetches PR title and description
   - Sends to LLM with prompt focused on documentation clarity
   - Posts results as a PR comment
4. Configure as non-blocking (informational only)
5. Include prompt version tracking metadata

**Integration with Existing CI Workflow:**
- Run GooseBot as a separate workflow parallel to existing CI
- Use the same checkout action as the existing workflow
- Ensure completion timing aligns with early CI checks

**Expected Outcome**: Contributors receive automated comments from "GooseBot" with suggestions to improve PR descriptions.

### Phase 2: Expanded Code Quality & Style Review
**Goal**: Extend AI review to analyze the code changes for correctness, best practices, and style.

**Implementation Steps:**
1. Enhance the existing workflow to analyze code diffs
2. Create functionality to:
   - Extract PR diff content (e.g., `git diff origin/main...HEAD`)
   - Break large diffs into manageable chunks
   - Send to LLM with expanded prompt covering code quality aspects
3. Structure the output as a readable PR comment with categorized feedback
4. Maintain as non-blocking (advisory)
5. Implement prompt optimizations to limit verbosity (top 3-5 issues)

**Expected Outcome**: GooseBot provides code-level feedback similar to a human reviewer, pointing out potential issues in a structured format.

### Phase 3: Multi-Agent Specialized Reviews
**Goal**: Improve depth by using separate LLM "agents" for different review aspects.

**Implementation Steps:**
1. Split the review into specialized aspects:
   - Documentation check
   - Code quality check
   - Performance check
   - Security check
   - Style check
2. Implement using matrix jobs or separate named jobs:
   ```yaml
   jobs:
     ai-multireview:
       runs-on: ubuntu-latest
       strategy:
         matrix:
           aspect: [ "docs", "quality", "performance", "security" ]
       steps:
         - uses: actions/checkout@v3
         - name: GooseBot Review for ${{ matrix.aspect }}
           run: python scripts/goosebot_review.py --aspect ${{ matrix.aspect }} --pr ${{ github.event.pull_request.number }}
   ```
3. Create specialized prompts for each aspect
4. Consolidate feedback into a well-structured report or post separate comments
5. Keep as advisory/non-blocking

**Expected Outcome**: GooseBot provides thorough multi-faceted reviews, similar to having specialized reviewers for different aspects.

### Phase 4: Refinement and Optional Enforcement
**Goal**: Fine-tune the system and optionally enforce certain checks.

**Implementation Steps:**
1. Add manual triggers for on-demand reviews
2. Implement severity classification for issues
3. Configure optional required checks:
   - Make critical issues optionally block merges
   - Add override mechanisms for maintainers
4. Establish continuous improvement process for prompts
5. Document the system for contributors
6. Support multiple LLM providers/models for comparison and fallback

**Expected Outcome**: GooseBot becomes a mature part of the CI pipeline with optional enforcement capabilities.

## Advanced Technical Implementation

### File Filtering System
GooseBot will include a robust file filtering system to control which files receive reviews:

- **Whitelist Patterns**: Comma-separated glob patterns to specify files that should be reviewed
  - Example: `*.rs,src/**/*.rs,lib/*.rs`
  - Default to `*` (all files) if not specified
  
- **Blacklist Patterns**: Comma-separated glob patterns to exclude files from review
  - Example: `tests/*,benches/*,examples/*`
  - Blacklist takes precedence over whitelist
  
- **Configuration**: Set via environment variables in the workflow
  ```yaml
  env:
    PR_REVIEW_WHITELIST: "*.rs,src/**/*.rs"
    PR_REVIEW_BLACKLIST: "tests/*,benches/*,vendor/*" 
  ```

- **Implementation**: Use a `FileFilterConfig` class to centralize filtering logic:
  ```python
  class FileFilterConfig:
      def __init__(self, whitelist_patterns, blacklist_patterns):
          self.whitelist_patterns = whitelist_patterns
          self.blacklist_patterns = blacklist_patterns
          
      @classmethod
      def from_env(cls):
          # Parse from environment variables
          
      def should_review_file(self, filename):
          # Check blacklist first (takes precedence)
          # Then check if matches any whitelist pattern
  ```

### Line Position Mapping
GooseBot will include advanced line position mapping to ensure review comments are placed at the correct locations:

- **Patch Analysis**: Parse Git patch format to map file line numbers to PR position values
  ```python
  def calculate_line_positions(patch):
      # Parse patch to map source lines to GitHub position values
      # Track hunk headers and context
      # Return a mapping of {line_number: position}
  ```

- **Closest Line Finding**: When exact line matches aren't available, find the closest appropriate line within a configurable distance
  ```python
  def find_closest_line(target_line, positions, max_distance=3):
      # Find the line in positions closest to target_line
      # Return None if no line is within max_distance
  ```

- **Position Tracking**: Maintain accurate position tracking for context and added lines in the diff

This will solve a common problem with PR reviews where comments can't be placed correctly due to line number mismatches between file content and what's visible in the PR.

### Comment Formatting
GooseBot will format review comments to maximize usefulness:

- **Inline Suggestions**: Use GitHub's suggestion format for proposed code changes:
  ```
  Description of the issue and why it should be improved

  ```suggestion
  The exact code that should replace this line
  ```
  ```
  
- **Summary Comment**: Create a comprehensive review summary including:
  - List of files reviewed
  - List of files skipped (with reason)
  - Count of suggestions made
  - General comments for issues not tied to specific lines

- **Implementation**: Structure the LLM prompt to return well-formatted suggestions:
  ```python
  prompt = f"""Review this code and respond with ONLY a JSON array of found issues. For each issue include:
  - line number
  - explanation of the issue
  - concrete code suggestion for improvement

  Format EXACTLY like this JSON array, with no other text:

  [
      {{
          "line": 1,
          "comment": "Description of the issue and why it should be improved",
          "suggestion": "The exact code that should replace this line"
      }}
  ]
  """
  ```

### Duplicate Comment Prevention
GooseBot will track existing comments to avoid duplication:

- **Comment Mapping**: Build a map of existing comments by path and position
  ```python
  def get_existing_comments(self):
      comments = self.pull_request.get_review_comments()
      existing = {}
      for comment in comments:
          key = f"{comment.path}:{comment.position}"
          existing[key] = comment.body
      return existing
  ```

- **Duplicate Check**: Before adding a new comment, check if a similar one already exists
  ```python
  comment_key = f"{file.filename}:{position}"
  if comment_key not in existing_comments:
      # Add the comment
  ```

- **Incremental Reviews**: Support incremental reviews that only comment on new issues

### Error Handling and Logging
GooseBot will implement robust error handling and logging:

- **Structured Logging**: Use Python's logging module for consistent, leveled logs
  ```python
  import logging
  logging.basicConfig(level=logging.INFO)
  logger = logging.getLogger(__name__)
  ```

- **Error Recovery**: Attempt to continue reviewing files even if one file fails
  ```python
  for file in changed_files:
      try:
          # Process file
      except Exception as e:
          logger.error(f"Error processing {file.filename}: {e}")
          continue  # Continue with next file
  ```

- **Debug Information**: Include detailed debug info for troubleshooting
  ```python
  logger.debug(f"Line positions map: {line_positions}")
  ```

- **Summary Statistics**: Log counts of files reviewed, comments made, and files skipped

### GooseBot Feedback Guidelines
To ensure that GooseBot provides helpful, focused feedback:

1. Use prompt engineering for clarity and brevity
   - Limit feedback to maximum 5 key points
   - Focus on significant issues
2. Ensure friendly, constructive tone
   - Use phrases like "Consider using…" instead of commands
3. Avoid redundancy with other tools
   - Don't duplicate linter/clippy warnings
4. Categorize feedback clearly
5. Implement length limits for large PRs
6. Establish human oversight during initial phases

### Modular Design
The implementation will be structured for reusability:

1. Store Goose-specific context in configuration
2. Parameterize project-specific criteria
3. Create generic, reusable scripts
4. Implement feature flags for different AI checks
5. Keep the system decoupled from Goose application code

### Testing Strategy
To ensure GooseBot's reliability, we'll implement a comprehensive testing approach:

1. **Unit Tests**:
   - Test file filtering logic
   - Test line position mapping
   - Test comment formatting
   - Test GitHub API interaction (with mocks)
   
2. **Integration Tests**:
   - Create test PR workflows that validate end-to-end functionality
   - Test with synthetic PR content
   - Mock LLM API responses for predictable testing
   
3. **Regression Tests**:
   - Maintain a collection of sample PRs with expected outcomes
   - Compare GooseBot output against expected baselines
   
4. **Self-Review Tests**:
   - GooseBot should review its own PRs for code quality
   - Implement a validation step to ensure PR comments are well-formed
   
5. **Implementation**:
   ```yaml
   name: GooseBot Self-Test
   on:
     pull_request:
       paths:
         - 'scripts/goosebot_*.py'
         - '.github/workflows/goosebot*.yml'
   jobs:
     test-goosebot:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Run GooseBot Unit Tests
           run: |
             python -m unittest discover -s scripts/tests
         - name: Run GooseBot Integration Tests
           run: |
             python scripts/tests/integration_test.py
   ```

### Prompt Version Control
To manage LLM prompts effectively:

1. **Prompt Repository**:
   - Store prompts as versioned template files
   - Include metadata (version, author, date, purpose)
   - Support multiple language models
   
2. **Version Tagging**:
   - Embed version information in each prompt
   - Record prompt version in review comments
   - Track prompt performance metrics by version
   
3. **A/B Testing**:
   - Support parallel prompt testing on similar PRs
   - Gather metrics on prompt effectiveness
   - Framework for comparing prompt versions
   
4. **Implementation**:
   ```python
   class PromptTemplate:
       def __init__(self, template_path, version, model):
           self.template_path = template_path
           self.version = version
           self.model = model
           self.metadata = {}
           self._load()
           
       def _load(self):
           with open(self.template_path, "r") as f:
               content = f.read()
               # Parse metadata from comments
               # Load template content
               
       def render(self, **kwargs):
           # Render template with parameters
           # Add version metadata
           # Return formatted prompt
   ```

### Feedback Loop Metrics
To measure and improve GooseBot's effectiveness:

1. **Feedback Collection**:
   - Add reaction buttons to GooseBot comments
   - Track when suggestions are accepted/rejected
   - Collect user feedback through slash commands
   
2. **Metrics Dashboard**:
   - Track suggestion acceptance rate
   - Measure comment quality and relevance
   - Monitor performance across different code areas
   
3. **Continuous Improvement**:
   - Use feedback to refine prompts
   - Adjust review focus based on what helps most
   - Regular review of metrics with maintainers
   
4. **Implementation**:
   ```python
   def track_feedback(comment_id, feedback_type):
       """Record feedback on a GooseBot comment"""
       # Store in database or log
       # Update metrics
       
   def process_reaction(reaction, comment_id):
       """Process GitHub reaction on comment"""
       if reaction == "+1":
           track_feedback(comment_id, "positive")
       elif reaction == "-1":
           track_feedback(comment_id, "negative")
   ```

### Cost Management
To control API usage costs effectively:

1. **Usage Monitoring**:
   - Track token usage per PR review
   - Set daily/monthly limits
   - Alert on unusual usage patterns
   
2. **Optimization Strategies**:
   - Chunk large PRs intelligently
   - Filter unnecessary files (e.g., generated code)
   - Cache LLM responses where applicable
   - Adjust verbosity based on PR size
   
3. **Budget Controls**:
   - Implement token caps per PR
   - Prioritize reviews based on PR size/importance
   - Support different review depths based on PR type
   
4. **Implementation**:
   ```python
   class TokenUsageTracker:
       def __init__(self, budget_limit):
           self.budget_limit = budget_limit
           self.current_usage = 0
           
       def can_process(self, estimated_tokens):
           return self.current_usage + estimated_tokens <= self.budget_limit
           
       def record_usage(self, prompt_tokens, completion_tokens):
           usage = prompt_tokens + completion_tokens
           self.current_usage += usage
           return self.current_usage
   ```

### Fallback Mechanisms
To ensure reliability when the LLM service is unavailable:

1. **Service Health Checks**:
   - Monitor LLM API availability
   - Check response times and quality
   - Detect truncated or malformed responses
   
2. **Multi-Provider Support**:
   - Support multiple LLM providers (Anthropic, OpenAI, etc.)
   - Implement provider rotation for reliability
   - Fallback to alternative models when primary is unavailable
   
3. **Graceful Degradation**:
   - Perform basic reviews without LLM if needed
   - Skip complex analysis but maintain simple checks
   - Clear error reporting when service is unavailable
   
4. **Implementation**:
   ```python
   class LLMProvider:
       def __init__(self, providers_config):
           self.providers = self._load_providers(providers_config)
           self.primary = providers_config.get("primary", "anthropic")
           
       def get_provider(self):
           """Get current working provider"""
           # Try primary first
           if self._check_health(self.primary):
               return self.providers[self.primary]
               
           # Try fallbacks in order
           for name, provider in self.providers.items():
               if name != self.primary and self._check_health(name):
                   logger.warning(f"Using fallback provider {name}")
                   return provider
                   
           # No working providers
           raise NoAvailableProviderError("All LLM providers unavailable")
   ```

### Security and Control Mechanisms
To ensure security and control costs:

1. Store API tokens in GitHub Secrets
2. Use least-privilege permissions
3. Implement trigger controls:
   - Label-based triggers (e.g., "ai-review")
   - Comment-based triggers (e.g., "/goosebot-review")
4. Set resource constraints to manage API usage
5. Ensure only public, non-sensitive data is sent to the LLM

### Gradual Integration Process
The roll-out will follow these best practices:

1. Start as non-blocking feedback
2. Monitor effectiveness and gather maintainer feedback
3. Introduce optional enforcement gates
4. Add full enforcement on opt-in basis
5. Document the process for all contributors

## Technical Implementation

### Required Components
1. GitHub Actions workflow definitions
2. Python script for LLM API interaction
3. Prompts for different review aspects
4. Comment formatting templates
5. Documentation updates

### Dependencies
1. GitHub API access via PyGithub package
2. Anthropic API access via anthropic package
3. Python environment with HTTP libraries
4. Additional packages:
   ```
   pip install anthropic==0.45.2 PyGithub==2.6.0
   ```

## Documentation Updates
The following documentation updates will be made:

1. Add this plan document to the memory-bank
2. Update `activeContext.md` to reference AI code reviews
3. Mention in `progress.md` under Roadmap
4. Update `techContext.md` to include in development workflow
5. Create contributor documentation on working with GooseBot

## Next Steps
1. Set up API access and tokens (Anthropic API key for internal hosted instance)
2. Implement Phase 1 (PR clarity)
   - Create Python script for PR analysis
   - Implement GitHub workflow file
   - Develop initial prompt templates
   - Test on sample PRs
3. Gather feedback from maintainers
4. Proceed to subsequent phases based on success
