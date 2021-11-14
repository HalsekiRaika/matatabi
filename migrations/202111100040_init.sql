-- DROP TABLE affiliations;
CREATE TABLE affiliations (
    affiliation_id VARCHAR(32) NOT NULL PRIMARY KEY,
    name VARCHAR(32) NOT NULL
);

-- DROP TABLE vtubers;
CREATE TABLE vtubers (
    vtuber_id VARCHAR(32) NOT NULL PRIMARY KEY,
    name VARCHAR(32) NOT NULL,
    affiliation VARCHAR(32),
    FOREIGN KEY (affiliation) REFERENCES affiliations(affiliation_id)
);

-- DROP TABLE channels;
CREATE TABLE channels (
    channel_id VARCHAR(32) NOT NULL PRIMARY KEY,
    vtuber_id VARCHAR(32),
    FOREIGN KEY (vtuber_id) REFERENCES vtubers(vtuber_id),
    logo_url VARCHAR(128) NOT NULL,
    published_at TIMESTAMP NOT NULL,
    description TEXT
);

-- DROP TABLE lives;
CREATE TABLE lives (
    video_id VARCHAR(11) NOT NULL PRIMARY KEY,
    channel_id VARCHAR(32), 
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    will_start_at TIMESTAMP NULL,
    started_at TIMESTAMP NULL,
    thumbnail_url VARCHAR(128)
);
