-- DROP TABLE affiliations;
CREATE TABLE affiliations (
    affiliation_id BIGSERIAL NOT NULL PRIMARY KEY,
    name VARCHAR(32) NOT NULL
);

-- DROP TABLE livers;
CREATE TABLE livers (
    liver_id BIGSERIAL NOT NULL PRIMARY KEY,
    affiliation_id BIGSERIAL,
    name VARCHAR(32) NOT NULL,
    localized_name VARCHAR(32) NOT NULL,

    FOREIGN KEY (affiliation_id) REFERENCES affiliations(affiliation_id)
);

-- DROP TABLE channels;
CREATE TABLE channels (
    -- Channel ID uses the Youtube identifier string.
    -- (Example: UCxxxxxxxxxxxxxxxxxxxxxx)
    channel_id VARCHAR(24) NOT NULL PRIMARY KEY,
    liver_id BIGSERIAL,
    logo_url VARCHAR(256) NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    description TEXT,

    FOREIGN KEY (liver_id) REFERENCES livers(liver_id)
);

-- DROP TABLE lives;
CREATE TABLE videos (
    -- VideoId uses the 11-character identifier
    -- string of Youtube as well as ChannelId.
    video_id VARCHAR(11) NOT NULL PRIMARY KEY,
    channel_id VARCHAR(24),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    published_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    will_start_at TIMESTAMPTZ NULL,
    started_at TIMESTAMPTZ NULL,
    thumbnail_url VARCHAR(128),

    FOREIGN KEY (channel_id) REFERENCES channels(channel_id)
);
