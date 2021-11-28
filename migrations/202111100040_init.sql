-- DROP TABLE affiliations;
CREATE TABLE affiliations (
    affiliation_id BIGSERIAL NOT NULL PRIMARY KEY,
    name VARCHAR(32) NOT NULL
);

-- DROP TABLE vtubers;
CREATE TABLE vtubers (
    vtuber_id BIGSERIAL NOT NULL PRIMARY KEY,
    affiliation BIGSERIAL,
      FOREIGN KEY (affiliation)
        REFERENCES affiliations(affiliation_id),
    name VARCHAR(32) NOT NULL
);

-- DROP TABLE channels;
CREATE TABLE channels (
    -- Channel ID uses the Youtube identifier string.
    -- (Example: UCxxxxxxxxxxxxxxxxxxxxxx)
    channel_id VARCHAR(24) NOT NULL PRIMARY KEY,
    vtuber_id BIGSERIAL,
      FOREIGN KEY (vtuber_id)
        REFERENCES vtubers(vtuber_id),
    logo_url VARCHAR(128) NOT NULL,
    published_at TIMESTAMP NOT NULL,
    description TEXT
);

-- DROP TABLE lives;
CREATE TABLE lives (
    -- VideoId uses the 11-character identifier
    -- string of Youtube as well as ChannelId.
    video_id VARCHAR(11) NOT NULL PRIMARY KEY,
    channel_id VARCHAR(24),
      FOREIGN KEY (channel_id)
        REFERENCES channels(channel_id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    will_start_at TIMESTAMP NULL,
    started_at TIMESTAMP NULL,
    thumbnail_url VARCHAR(128)
);
