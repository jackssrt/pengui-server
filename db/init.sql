CREATE TABLE
	IF NOT EXISTS playerMinigameScores (
		uuid varchar(255) NOT NULL PRIMARY KEY,
		game varchar(255) NOT NULL,
		minigameId int NOT NULL,
		score int NOT NULL,
		timestampCompleted timestamp NOT NULL
	);

CREATE TABLE
	IF NOT EXISTS playerGameData ( -- pgd
		uuid varchar(255) NOT NULL PRIMARY KEY,
		game varchar(255) NOT NULL,
		online BOOLEAN NOT NULL,
		timestampLastActive timestamp,
		medalCountBronze INT(255) NOT NULL DEFAULT 0,
		medalCountSilver INT(255) NOT NULL DEFAULT 0,
		medalCountGold INT(255) NOT NULL DEFAULT 0,
		medalCountPlatinum INT(255) NOT NULL DEFAULT 0,
		medalCountDiamond INT(255) NOT NULL DEFAULT 0
	);

CREATE TABLE
	IF NOT EXISTS chatMessages (
		msgId VARCHAR(255) NOT NULL PRIMARY KEY,
		game VARCHAR(255) NOT NULL,
		uuid VARCHAR(255) NOT NULL,
		mapId VARCHAR(255),
		prevMapId VARCHAR(255),
		prevLocations VARCHAR(255),
		x INT (255),
		y INT (255),
		contents TEXT NOT NULL,
		timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
	);

CREATE TABLE
	IF NOT EXISTS accounts (
		uuid VARCHAR(255) NOT NULL PRIMARY KEY,
		user VARCHAR(255) NOT NULL UNIQUE KEY, -- username
		badge VARCHAR(255)
	);

CREATE TABLE
	IF NOT EXISTS players ( -- pd
		uuid VARCHAR(255) NOT NULL PRIMARY KEY,
		`rank` BIT (8) NOT NULL DEFAULT FALSE,
		banned BOOLEAN NOT NULL DEFAULT FALSE,
		muted BOOLEAN NOT NULL DEFAULT FALSE,
		ip VARCHAR(255) NOT NULL UNIQUE KEY
	);

CREATE TABLE
	IF NOT EXISTS playerSessions ( -- ps
		uuid VARCHAR(255) NOT NULL PRIMARY KEY,
		sessionId VARCHAR(255) NOT NULL UNIQUE KEY,
		expiration TIMESTAMP NOT NULL
	);

CREATE TABLE
	IF NOT EXISTS parties ( -- p
		id VARCHAR(255) NOT NULL PRIMARY KEY,
		game VARCHAR(255) NOT NULL
	);

CREATE TABLE
	IF NOT EXISTS partyMembers ( -- pm
		partyId VARCHAR(255) NOT NULL PRIMARY KEY,
		uuid VARCHAR(255) NOT NULL UNIQUE KEY
	);

CREATE TABLE
	IF NOT EXISTS playerBlocks ( -- pb
		uuid VARCHAR(255) NOT NULL KEY,
		targetUuid VARCHAR(255) NOT NULL,
		timestamp TIMESTAMP NOT NULL
	)