CREATE TABLE IF NOT EXISTS link_clicks (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    link_id BIGINT NOT NULL REFERENCES links(id) ON DELETE CASCADE,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    referer TEXT NULL,
    user_agent TEXT NULL,
    ip INET NULL
);

CREATE INDEX IF NOT EXISTS link_clicks_link_id_clicked_at_idx
    ON link_clicks (link_id, clicked_at);
