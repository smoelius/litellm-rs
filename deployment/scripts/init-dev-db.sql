-- Development Database Initialization
-- This script sets up the development database with sample data

-- Run the main initialization script first
\i /docker-entrypoint-initdb.d/init-db.sql

-- Insert sample users for development
INSERT INTO users (username, email, password_hash, role) VALUES 
('developer', 'dev@localhost', '$argon2id$v=19$m=65536,t=3,p=4$DevHashHere', 'admin'),
('testuser', 'test@localhost', '$argon2id$v=19$m=65536,t=3,p=4$TestHashHere', 'user'),
('readonly', 'readonly@localhost', '$argon2id$v=19$m=65536,t=3,p=4$ReadonlyHashHere', 'readonly')
ON CONFLICT (username) DO NOTHING;

-- Insert sample teams
INSERT INTO teams (name, description) VALUES 
('Development Team', 'Team for developers'),
('QA Team', 'Quality assurance team'),
('Demo Team', 'Team for demonstrations')
ON CONFLICT (name) DO NOTHING;

-- Add users to teams
INSERT INTO user_teams (user_id, team_id, role) 
SELECT u.id, t.id, 'admin'
FROM users u, teams t 
WHERE u.username = 'developer' AND t.name = 'Development Team'
ON CONFLICT DO NOTHING;

INSERT INTO user_teams (user_id, team_id, role) 
SELECT u.id, t.id, 'member'
FROM users u, teams t 
WHERE u.username = 'testuser' AND t.name = 'QA Team'
ON CONFLICT DO NOTHING;

-- Insert sample API keys
INSERT INTO api_keys (id, name, key_hash, user_id, permissions) 
SELECT 
    'dev_key_001',
    'Development Key',
    'dev_key_hash_001',
    u.id,
    '["chat:create", "completion:create", "embedding:create", "model:list"]'::jsonb
FROM users u 
WHERE u.username = 'developer'
ON CONFLICT (id) DO NOTHING;

INSERT INTO api_keys (id, name, key_hash, user_id, permissions) 
SELECT 
    'test_key_001',
    'Test Key',
    'test_key_hash_001',
    u.id,
    '["chat:create", "model:list"]'::jsonb
FROM users u 
WHERE u.username = 'testuser'
ON CONFLICT (id) DO NOTHING;

-- Insert sample provider health data
INSERT INTO provider_health (provider_name, is_healthy, response_time_ms) VALUES 
('openai', true, 250),
('anthropic', true, 300),
('azure', false, 5000),
('google', true, 200)
ON CONFLICT (provider_name) DO UPDATE SET
    is_healthy = EXCLUDED.is_healthy,
    response_time_ms = EXCLUDED.response_time_ms,
    last_check = NOW();

-- Insert sample request logs for testing
INSERT INTO request_logs (
    request_id, user_id, provider, model, endpoint, method, 
    status_code, prompt_tokens, completion_tokens, total_tokens, 
    cost_usd, response_time_ms
) 
SELECT 
    'req_' || generate_random_uuid()::text,
    u.id,
    'openai',
    'gpt-3.5-turbo',
    '/v1/chat/completions',
    'POST',
    200,
    50,
    25,
    75,
    0.0015,
    250
FROM users u 
WHERE u.username = 'developer'
LIMIT 10;

-- Insert sample usage statistics
INSERT INTO usage_statistics (user_id, provider, model, date, request_count, token_count, cost_usd)
SELECT 
    u.id,
    'openai',
    'gpt-3.5-turbo',
    CURRENT_DATE - INTERVAL '1 day' * generate_series(0, 6),
    (random() * 100)::integer + 10,
    (random() * 10000)::integer + 1000,
    (random() * 10)::numeric(10,6) + 1
FROM users u 
WHERE u.username IN ('developer', 'testuser');

-- Create some test data for analytics
DO $$
DECLARE
    dev_user_id BIGINT;
    test_user_id BIGINT;
    i INTEGER;
BEGIN
    -- Get user IDs
    SELECT id INTO dev_user_id FROM users WHERE username = 'developer';
    SELECT id INTO test_user_id FROM users WHERE username = 'testuser';
    
    -- Insert sample request logs for the past week
    FOR i IN 1..50 LOOP
        INSERT INTO request_logs (
            request_id, user_id, provider, model, endpoint, method,
            status_code, prompt_tokens, completion_tokens, total_tokens,
            cost_usd, response_time_ms, created_at
        ) VALUES (
            'req_dev_' || i,
            dev_user_id,
            CASE (i % 3)
                WHEN 0 THEN 'openai'
                WHEN 1 THEN 'anthropic'
                ELSE 'google'
            END,
            CASE (i % 3)
                WHEN 0 THEN 'gpt-3.5-turbo'
                WHEN 1 THEN 'claude-3-sonnet'
                ELSE 'gemini-pro'
            END,
            '/v1/chat/completions',
            'POST',
            CASE WHEN i % 10 = 0 THEN 429 ELSE 200 END, -- 10% rate limit errors
            (random() * 100)::integer + 10,
            (random() * 50)::integer + 5,
            (random() * 150)::integer + 15,
            (random() * 0.01)::numeric(10,6) + 0.001,
            (random() * 1000)::integer + 100,
            NOW() - INTERVAL '1 hour' * (random() * 168) -- Past week
        );
    END LOOP;
    
    -- Insert some test user requests
    FOR i IN 1..20 LOOP
        INSERT INTO request_logs (
            request_id, user_id, provider, model, endpoint, method,
            status_code, prompt_tokens, completion_tokens, total_tokens,
            cost_usd, response_time_ms, created_at
        ) VALUES (
            'req_test_' || i,
            test_user_id,
            'openai',
            'gpt-3.5-turbo',
            '/v1/chat/completions',
            'POST',
            200,
            (random() * 50)::integer + 5,
            (random() * 25)::integer + 2,
            (random() * 75)::integer + 7,
            (random() * 0.005)::numeric(10,6) + 0.0005,
            (random() * 500)::integer + 50,
            NOW() - INTERVAL '1 hour' * (random() * 72) -- Past 3 days
        );
    END LOOP;
END $$;

-- Create development-specific views
CREATE OR REPLACE VIEW dev_daily_stats AS
SELECT 
    DATE(created_at) as date,
    provider,
    COUNT(*) as requests,
    AVG(response_time_ms) as avg_response_time,
    SUM(total_tokens) as total_tokens,
    SUM(cost_usd) as total_cost,
    COUNT(CASE WHEN status_code >= 400 THEN 1 END) as errors
FROM request_logs 
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
GROUP BY DATE(created_at), provider
ORDER BY date DESC, provider;

-- Print setup completion message
DO $$
BEGIN
    RAISE NOTICE 'Development database initialized successfully!';
    RAISE NOTICE 'Sample users created:';
    RAISE NOTICE '  - developer (admin) - password: dev123';
    RAISE NOTICE '  - testuser (user) - password: test123';
    RAISE NOTICE '  - readonly (readonly) - password: readonly123';
    RAISE NOTICE 'Sample API keys created for testing';
    RAISE NOTICE 'Sample data inserted for analytics testing';
END $$;
