# AI Assistant Customization

**Purpose:** Configure and customize the AI assistant for your hackathon.

---

## Overview

The AI Assistant provides:
- **Chat** - Answer participant questions
- **Idea Generator** - Suggest project ideas
- **Team Matcher** - Help users find teammates
- **Code Review** - Provide feedback on code
- **Organizer Insights** - Analytics for admins

**Tech Stack:**
- LangChain for LLM orchestration
- pgvector for knowledge base embeddings
- RAG (Retrieval-Augmented Generation) for custom knowledge

---

## Configuration

### Enable/Disable Features

Edit `.env`:
```bash
# Main toggle
AI_ENABLED=true

# Individual features
AI_CHAT=true
AI_IDEA_GENERATOR=true
AI_TEAM_MATCHER=false
AI_CODE_REVIEW=false
AI_ORGANIZER_INSIGHTS=true
```

---

### LLM Provider Configuration

**OpenAI (Default):**
```bash
AI_PROVIDER=openai
OPENAI_API_KEY=sk-...
AI_MODEL=gpt-4-turbo
AI_MAX_TOKENS=1024
AI_TEMPERATURE=0.7
```

**Anthropic:**
```bash
AI_PROVIDER=anthropic
ANTHROPIC_API_KEY=sk-ant-...
AI_MODEL=claude-3-opus-20240229
AI_MAX_TOKENS=1024
AI_TEMPERATURE=0.7
```

**Local (Ollama):**
```bash
AI_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
AI_MODEL=llama2:7b
AI_MAX_TOKENS=1024
AI_TEMPERATURE=0.7
```

---

## Knowledge Base

### Upload Custom Knowledge

**Via Admin Dashboard:**

1. Navigate to `/dashboard/admin/ai`
2. Click "Upload Knowledge"
3. Select files (PDF, TXT, MD supported)
4. Add metadata (title, category, tags)
5. Click "Upload"

**Via API:**

```bash
curl -X POST http://localhost:8000/api/ai/admin/knowledge \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -F "file=@/path/to/document.pdf" \
  -F "title=Hackathon Rules" \
  -F "category=rules" \
  -F "tags=hackathon,rules,guidelines"
```

---

### Knowledge Categories

Organize knowledge into categories:

| Category | Description | Example Content |
|----------|-------------|-----------------|
| `rules` | Hackathon rules | Code of conduct, eligibility |
| `faq` | Frequently asked questions | Common questions and answers |
| `schedule` | Event schedule | Timeline, workshop times |
| `resources` | Learning resources | Tutorials, documentation links |
| `sponsors` | Sponsor information | Company descriptions, prizes |
| `technical` | Technical guides | API docs, deployment guides |

---

### Manage Knowledge Base

**List Knowledge:**
```bash
curl http://localhost:8000/api/ai/admin/knowledge \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Delete Knowledge:**
```bash
curl -X DELETE http://localhost:8000/api/ai/admin/knowledge/KNOWLEDGE_ID \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Update Knowledge:**
```bash
curl -X PUT http://localhost:8000/api/ai/admin/knowledge/KNOWLEDGE_ID \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -d '{
    "title": "Updated Title",
    "tags": ["new", "tags"]
  }'
```

---

## Customizing AI Behavior

### System Prompts

Edit `services/ai/src/prompts/system.py`:

```python
SYSTEM_PROMPT = """
You are the AI assistant for {hackathon_name}, a hackathon taking place on {hackathon_dates}.

Your role is to:
1. Answer participant questions about the hackathon
2. Help users generate project ideas
3. Assist with team formation
4. Provide code review feedback (if enabled)

Tone: Friendly, encouraging, and helpful
Style: Concise but informative
Audience: Hackathon participants (beginners to advanced)

Rules:
- Always be positive and supportive
- Encourage creativity and experimentation
- Remind users about the code of conduct
- Direct technical questions to appropriate resources
- Don't provide complete solutions, but guide users to discover answers

If you don't know something, say so and suggest where they might find the answer.
"""
```

---

### Feature-Specific Prompts

**Idea Generator Prompt:**
```python
IDEA_GENERATOR_PROMPT = """
Help the user brainstorm project ideas for {hackathon_name}.

Consider:
- The hackathon theme: {theme}
- Available APIs and technologies: {technologies}
- Skill level of the user: {skill_level}

Generate 3-5 creative project ideas that:
1. Can be built in {duration} hours
2. Use relevant technologies
3. Solve a real problem or are fun/creative

For each idea, provide:
- Project name
- One-sentence description
- Key technologies
- Difficulty level (1-5)
"""
```

**Team Matcher Prompt:**
```python
TEAM_MATCHER_PROMPT = """
Help the user find potential teammates for {hackathon_name}.

User profile:
- Skills: {user_skills}
- Interests: {user_interests}
- Looking for: {looking_for}

Suggest 3-5 types of teammates that would complement this user:
1. Role/skills needed
2. Why this combination works well
3. Example project they could build together

Also provide tips for effective team formation.
"""
```

**Code Review Prompt:**
```python
CODE_REVIEW_PROMPT = """
Review the following code for a hackathon project.

Provide feedback on:
1. Code quality (readability, organization)
2. Potential bugs or issues
3. Security concerns
4. Performance considerations
5. Suggestions for improvement

Be constructive and encouraging. Remember this is a hackathon project
built under time constraints.

Prioritize feedback by:
- Critical: Security issues, major bugs
- Important: Code quality, maintainability
- Nice-to-have: Optimizations, style improvements

Code:
{code}
"""
```

---

## RAG Configuration

### Embedding Model

Edit `services/ai/src/config.py`:

```python
# Embedding model for knowledge base
EMBEDDING_MODEL = "text-embedding-3-small"  # OpenAI
# or
EMBEDDING_MODEL = "all-MiniLM-L6-v2"  # Local (sentence-transformers)

# Chunk size for document splitting
CHUNK_SIZE = 500  # tokens
CHUNK_OVERLAP = 50  # tokens

# Number of relevant chunks to retrieve
RETRIEVAL_TOP_K = 5
```

---

### Vector Store Configuration

```python
# PostgreSQL with pgvector
VECTOR_STORE_TYPE = "postgres"
PGVECTOR_INDEX = "hnsw"  # or "ivfflat"
PGVECTOR_LISTS = 100  # for ivfflat
PGVECTOR_PROBES = 10  # for ivfflat

# Similarity threshold (0-1, higher = more strict)
SIMILARITY_THRESHOLD = 0.7
```

---

## Fine-tuning (Advanced)

### Collect Training Data

Export conversation logs:
```bash
curl http://localhost:8000/api/ai/admin/conversations \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  --output conversations.json
```

### Create Fine-tuning Dataset

Format for OpenAI fine-tuning:
```json
[
  {
    "messages": [
      {"role": "system", "content": "You are a helpful hackathon assistant."},
      {"role": "user", "content": "How do I submit my project?"},
      {"role": "assistant", "content": "To submit your project, go to the dashboard..."}
    ]
  }
]
```

### Upload for Fine-tuning

```bash
openai api files.create \
  -f fine-tuning-data.jsonl \
  -p fine-tune

openai api fine_tunes.create \
  -t <file-id> \
  -m davinci
```

---

## Monitoring AI

### Conversation Analytics

**Dashboard:** `/dashboard/admin/ai`

**Metrics:**
- Total conversations
- Conversations per day
- Average conversation length
- Most common questions
- User satisfaction (thumbs up/down)

---

### Conversation Logs

**View Logs:**
```bash
curl http://localhost:8000/api/ai/admin/conversations?limit=50 \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Filter by User:**
```bash
curl "http://localhost:8000/api/ai/admin/conversations?user_id=USER_ID" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Search Messages:**
```bash
curl "http://localhost:8000/api/ai/admin/conversations/search?q=deployment" \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

---

### Quality Assurance

**Review Conversations:**
1. Navigate to `/dashboard/admin/ai`
2. Click "Conversation Logs"
3. Review flagged conversations
4. Rate response quality
5. Provide feedback for improvement

**Common Issues to Watch:**
- Incorrect information
- Inappropriate responses
- Repetitive answers
- Failure to use knowledge base

---

## Best Practices

### 1. Start Small

Begin with just chat enabled, then add features gradually.

### 2. Curate Knowledge Base

- Upload high-quality, accurate documents
- Regularly update with new information
- Remove outdated content
- Organize with clear categories

### 3. Monitor and Iterate

- Review conversation logs weekly
- Collect user feedback
- Update prompts based on common issues
- Retrain fine-tuned models monthly

### 4. Set Expectations

Make it clear to users that:
- AI is a helper, not official support
- Responses may not always be accurate
- They should verify important information
- Human support is available

### 5. Rate Limit AI Endpoints

Prevent abuse:
```yaml
# kong.yml
- name: ai-svc
  plugins:
    - rate-limiting:
        minute: 30  # 30 requests per minute per user
```

---

## Troubleshooting

### AI Not Responding

**Check:**
1. `AI_ENABLED=true` in `.env`
2. LLM API key is valid
3. AI service is healthy: `curl http://localhost:3007/health`
4. Check logs: `docker compose logs ai-svc`

### Knowledge Base Not Used

**Check:**
1. Documents uploaded and processed
2. Embeddings generated (check `ai.knowledge` table)
3. `RETRIEVAL_TOP_K > 0`
4. Similarity threshold not too high

### Slow Responses

**Solutions:**
1. Reduce `AI_MAX_TOKENS`
2. Use faster model (e.g., `gpt-3.5-turbo`)
3. Reduce `RETRIEVAL_TOP_K`
4. Enable caching for common questions

### Inaccurate Responses

**Solutions:**
1. Improve knowledge base quality
2. Refine system prompt
3. Lower `AI_TEMPERATURE` (more deterministic)
4. Add more specific examples to prompts

---

**Last Updated:** May 13, 2026  
**See Also:** [AI Assistant Service](../../PROJECT.md#ai-assistant-service), [Admin Dashboard](../../PROJECT.md#admin-dashboard)
