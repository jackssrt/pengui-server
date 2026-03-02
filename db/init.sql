-- MariaDB dump 10.19  Distrib 10.11.6-MariaDB, for debian-linux-gnu (x86_64)
--
-- Host: localhost    Database: ynodb
-- ------------------------------------------------------
-- Server version	10.11.6-MariaDB-0+deb12u1
/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;

/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;

/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;

/*!40101 SET NAMES utf8mb4 */;

/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;

/*!40103 SET TIME_ZONE='+00:00' */;

/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;

/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;

/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;

/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `2kkiApiQueries`
--
DROP TABLE IF EXISTS `2kkiApiQueries`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`2kkiApiQueries` (
		`action` varchar(32) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`query` varchar(500) CHARACTER
		SET
			utf8mb3 COLLATE utf8mb3_unicode_ci NOT NULL,
			`response` text CHARACTER
		SET
			utf8mb3 COLLATE utf8mb3_unicode_ci NOT NULL,
			`timestampExpired` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
			PRIMARY KEY (`action`, `query`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `accounts`
--
DROP TABLE IF EXISTS `accounts`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`accounts` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`ip` varchar(39) CHARACTER
		SET
			ascii COLLATE ascii_general_ci DEFAULT NULL,
			`user` varchar(12) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`pass` char(60) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`timestampRegistered` timestamp NULL DEFAULT NULL,
			`timestampLoggedIn` timestamp NULL DEFAULT NULL,
			`badge` varchar(32) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL DEFAULT 'null',
			`badgeSlotRows` int (11) NOT NULL DEFAULT 1,
			`badgeSlotCols` int (11) NOT NULL DEFAULT 3,
			`screenshotLimit` int (11) NOT NULL DEFAULT 10,
			`inactive` tinyint (1) NOT NULL DEFAULT 0,
			PRIMARY KEY (`uuid`),
			UNIQUE KEY `user` (`user`),
			KEY `inactive` (`inactive`),
			CONSTRAINT `accounts_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `badges`
--
DROP TABLE IF EXISTS `badges`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`badges` (
		`badgeId` varchar(32) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`game` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`hidden` tinyint (1) NOT NULL,
			`bp` int (11) NOT NULL DEFAULT 0,
			`percentUnlocked` float NOT NULL DEFAULT 0,
			PRIMARY KEY (`badgeId`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `bannedIpRanges`
--
DROP TABLE IF EXISTS `bannedIpRanges`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`bannedIpRanges` (
		`ipRange` varchar(39) NOT NULL,
		`name` varchar(32) DEFAULT NULL,
		PRIMARY KEY (`ipRange`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `chatMessages`
--
DROP TABLE IF EXISTS `chatMessages`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`chatMessages` (
		`msgId` char(12) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`game` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`contents` varchar(150) NOT NULL,
			`mapId` char(4) CHARACTER
		SET
			ascii COLLATE ascii_general_ci DEFAULT NULL,
			`prevMapId` char(4) CHARACTER
		SET
			ascii COLLATE ascii_general_ci DEFAULT NULL,
			`prevLocations` varchar(512) CHARACTER
		SET
			utf8mb3 COLLATE utf8mb3_unicode_ci DEFAULT NULL,
			`x` int (11) NOT NULL DEFAULT 0,
			`y` int (11) NOT NULL DEFAULT 0,
			`partyId` int (11) DEFAULT NULL,
			`timestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
			PRIMARY KEY (`msgId`),
			KEY `uuid` (`uuid`),
			KEY `partyId` (`partyId`),
			CONSTRAINT `chatMessages_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`) ON DELETE CASCADE,
			CONSTRAINT `chatMessages_ibfk_2` FOREIGN KEY (`partyId`) REFERENCES `parties` (`id`) ON DELETE CASCADE
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `eventCompletions`
--
DROP TABLE IF EXISTS `eventCompletions`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`eventCompletions` (
		`eventId` int (11) NOT NULL,
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`timestampCompleted` timestamp NULL DEFAULT NULL,
			`exp` int (11) NOT NULL,
			`type` int (11) NOT NULL DEFAULT 0,
			PRIMARY KEY (`eventId`, `type`, `uuid`),
			KEY `eventId` (`eventId`),
			KEY `uuid` (`uuid`),
			KEY `idx_timestampCompleted` (`timestampCompleted`),
			CONSTRAINT `eventCompletions_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `eventLocations`
--
DROP TABLE IF EXISTS `eventLocations`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`eventLocations` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`locationId` int (11) DEFAULT NULL,
		`gamePeriodId` int (11) NOT NULL,
		`type` int (11) NOT NULL,
		`startDate` date NOT NULL,
		`endDate` date NOT NULL,
		`exp` int (11) NOT NULL,
		PRIMARY KEY (`id`),
		KEY `gamePeriodId` (`gamePeriodId`),
		KEY `locationId` (`locationId`),
		CONSTRAINT `eventLocations_ibfk_2` FOREIGN KEY (`gamePeriodId`) REFERENCES `gameEventPeriods` (`id`),
		CONSTRAINT `eventLocations_ibfk_3` FOREIGN KEY (`locationId`) REFERENCES `gameLocations` (`id`)
	) ENGINE = InnoDB AUTO_INCREMENT = 1755 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `eventPeriods`
--
DROP TABLE IF EXISTS `eventPeriods`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`eventPeriods` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`startDate` date NOT NULL,
		`endDate` date NOT NULL,
		`periodOrdinal` int (11) NOT NULL,
		`enableVms` tinyint (1) NOT NULL DEFAULT 0,
		PRIMARY KEY (`id`),
		KEY `idx_periodOrdinal` (`periodOrdinal`)
	) ENGINE = InnoDB AUTO_INCREMENT = 10 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `eventVms`
--
DROP TABLE IF EXISTS `eventVms`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`eventVms` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`gamePeriodId` int (11) NOT NULL,
		`mapId` varchar(4) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`eventId` varchar(4) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`startDate` date NOT NULL,
			`endDate` date NOT NULL,
			`exp` int (11) NOT NULL,
			PRIMARY KEY (`id`),
			KEY `eventVms_ibfk_1` (`gamePeriodId`),
			CONSTRAINT `eventVms_ibfk_1` FOREIGN KEY (`gamePeriodId`) REFERENCES `gameEventPeriods` (`id`)
	) ENGINE = InnoDB AUTO_INCREMENT = 281 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `gameEventPeriods`
--
DROP TABLE IF EXISTS `gameEventPeriods`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`gameEventPeriods` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`periodId` int (11) NOT NULL,
		`game` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`enableVms` tinyint (1) NOT NULL DEFAULT 0,
			PRIMARY KEY (`id`),
			KEY `periodId` (`periodId`),
			CONSTRAINT `gameEventPeriods_ibfk_1` FOREIGN KEY (`periodId`) REFERENCES `eventPeriods` (`id`)
	) ENGINE = InnoDB AUTO_INCREMENT = 64 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `gameLocations`
--
DROP TABLE IF EXISTS `gameLocations`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`gameLocations` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`game` varchar(16) NOT NULL,
		`title` varchar(255) CHARACTER
		SET
			utf8mb3 COLLATE utf8mb3_unicode_ci NOT NULL,
			`titleJP` varchar(255) CHARACTER
		SET
			utf8mb3 COLLATE utf8mb3_unicode_ci DEFAULT NULL,
			`depth` int (11) NOT NULL,
			`minDepth` int (11) NOT NULL DEFAULT 0,
			`mapIds` text DEFAULT NULL,
			`secret` tinyint (1) NOT NULL DEFAULT 0,
			PRIMARY KEY (`id`),
			UNIQUE KEY `game` (`game`, `title`)
	) ENGINE = InnoDB AUTO_INCREMENT = 178485 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `gamePlayerCounts`
--
DROP TABLE IF EXISTS `gamePlayerCounts`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`gamePlayerCounts` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`game` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`playerCount` int (11) NOT NULL,
			PRIMARY KEY (`id`)
	) ENGINE = InnoDB AUTO_INCREMENT = 26303 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `parties`
--
DROP TABLE IF EXISTS `parties`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`parties` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`game` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`owner` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`name` varchar(255) NOT NULL,
			`public` tinyint (1) NOT NULL,
			`theme` varchar(32) CHARACTER
		SET
			utf8mb3 COLLATE utf8mb3_unicode_ci NOT NULL,
			`description` text NOT NULL,
			`pass` varchar(255) CHARACTER
		SET
			utf8mb4 COLLATE utf8mb4_bin NOT NULL,
			PRIMARY KEY (`id`),
			KEY `owner` (`owner`),
			CONSTRAINT `parties_ibfk_1` FOREIGN KEY (`owner`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB AUTO_INCREMENT = 19401 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `partyMembers`
--
DROP TABLE IF EXISTS `partyMembers`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`partyMembers` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`partyId` int (11) NOT NULL,
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			PRIMARY KEY (`partyId`, `uuid`),
			UNIQUE KEY `id` (`id`),
			KEY `uuid` (`uuid`),
			CONSTRAINT `partyMembers_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
			CONSTRAINT `partyMembers_ibfk_2` FOREIGN KEY (`partyId`) REFERENCES `parties` (`id`)
	) ENGINE = InnoDB AUTO_INCREMENT = 65698 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerBadges`
--
DROP TABLE IF EXISTS `playerBadges`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerBadges` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`badgeId` varchar(32) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`timestampUnlocked` timestamp NOT NULL DEFAULT current_timestamp(),
			`slotId` int (11) DEFAULT 0,
			`slotRow` int (11) NOT NULL DEFAULT 0,
			`slotCol` int (11) NOT NULL DEFAULT 0,
			PRIMARY KEY (`uuid`, `badgeId`),
			CONSTRAINT `playerBadges_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerBlocks`
--
DROP TABLE IF EXISTS `playerBlocks`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerBlocks` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`targetUuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`timestamp` timestamp NOT NULL,
			PRIMARY KEY (`uuid`, `targetUuid`),
			KEY `uuid` (`uuid`),
			KEY `targetUuid` (`targetUuid`),
			CONSTRAINT `playerMutes_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
			CONSTRAINT `playerMutes_ibfk_2` FOREIGN KEY (`targetUuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerEventLocationQueue`
--
DROP TABLE IF EXISTS `playerEventLocationQueue`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerEventLocationQueue` (
		`game` varchar(16) NOT NULL,
		`date` date NOT NULL,
		`queueIndex` int (11) NOT NULL,
		`locationId` int (11) NOT NULL,
		PRIMARY KEY (`game`, `date`, `queueIndex`),
		KEY `locationId` (`locationId`),
		CONSTRAINT `playerEventLocationQueue_ibfk_1` FOREIGN KEY (`locationId`) REFERENCES `gameLocations` (`id`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerEventLocations`
--
DROP TABLE IF EXISTS `playerEventLocations`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerEventLocations` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`locationId` int (11) DEFAULT NULL,
		`gamePeriodId` int (11) NOT NULL,
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`startDate` date NOT NULL,
			`endDate` date NOT NULL,
			PRIMARY KEY (`id`),
			KEY `uuid` (`uuid`),
			KEY `gamePeriodId` (`gamePeriodId`),
			KEY `locationId` (`locationId`),
			CONSTRAINT `playerEventLocations_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
			CONSTRAINT `playerEventLocations_ibfk_2` FOREIGN KEY (`gamePeriodId`) REFERENCES `gameEventPeriods` (`id`),
			CONSTRAINT `playerEventLocations_ibfk_3` FOREIGN KEY (`locationId`) REFERENCES `gameLocations` (`id`)
	) ENGINE = InnoDB AUTO_INCREMENT = 1182121 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerFriends`
--
DROP TABLE IF EXISTS `playerFriends`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerFriends` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`targetUuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`accepted` tinyint (1) NOT NULL DEFAULT 0,
			PRIMARY KEY (`uuid`, `targetUuid`),
			KEY `uuid` (`uuid`),
			KEY `targetUuid` (`targetUuid`),
			CONSTRAINT `playerFriends_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
			CONSTRAINT `playerFriends_ibfk_2` FOREIGN KEY (`targetUuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerGameData`
--
DROP TABLE IF EXISTS `playerGameData`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerGameData` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`game` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`name` varchar(12) NOT NULL DEFAULT '',
			`systemName` varchar(32) NOT NULL DEFAULT '',
			`spriteName` varchar(32) NOT NULL DEFAULT '',
			`spriteIndex` tinyint (8) NOT NULL DEFAULT 0,
			`medalCountBronze` tinyint (100) DEFAULT 0,
			`medalCountSilver` tinyint (100) DEFAULT 0,
			`medalCountGold` tinyint (100) DEFAULT 0,
			`medalCountPlatinum` tinyint (100) DEFAULT 0,
			`medalCountDiamond` tinyint (100) DEFAULT 0,
			`online` tinyint (1) NOT NULL DEFAULT 0,
			`timestampLastActive` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
			`lastGlobalMsgId` char(12) DEFAULT NULL,
			`lastPartyMsgId` char(12) DEFAULT NULL,
			PRIMARY KEY (`uuid`, `game`),
			CONSTRAINT `playerGameData_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerGameLocations`
--
DROP TABLE IF EXISTS `playerGameLocations`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerGameLocations` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`locationId` int (11) NOT NULL,
			`timestamp` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
			PRIMARY KEY (`uuid`, `locationId`),
			KEY `uuid` (`uuid`),
			KEY `locationId` (`locationId`),
			CONSTRAINT `playerGameLocations_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
			CONSTRAINT `playerGameLocations_ibfk_2` FOREIGN KEY (`locationId`) REFERENCES `gameLocations` (`id`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerMinigameScores`
--
DROP TABLE IF EXISTS `playerMinigameScores`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerMinigameScores` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`minigameId` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`score` int (11) NOT NULL,
			`timestampCompleted` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
			`game` varchar(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			PRIMARY KEY (`uuid`, `minigameId`),
			CONSTRAINT `playerMinigameScores_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerScreenshotLikes`
--
DROP TABLE IF EXISTS `playerScreenshotLikes`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerScreenshotLikes` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`screenshotId` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`timestamp` timestamp NOT NULL DEFAULT current_timestamp(),
			PRIMARY KEY (`uuid`, `screenshotId`),
			KEY `uuid` (`uuid`),
			KEY `screenshotId` (`screenshotId`),
			CONSTRAINT `playerScreenshotLikes_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
			CONSTRAINT `playerScreenshotLikes_ibfk_2` FOREIGN KEY (`screenshotId`) REFERENCES `playerScreenshots` (`id`) ON DELETE CASCADE
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerScreenshots`
--
DROP TABLE IF EXISTS `playerScreenshots`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerScreenshots` (
		`id` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`game` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`mapId` char(4) NOT NULL DEFAULT '0000',
			`mapX` int (11) NOT NULL DEFAULT 0,
			`mapY` int (11) NOT NULL DEFAULT 0,
			`timestamp` timestamp NOT NULL DEFAULT current_timestamp(),
			`public` tinyint (1) NOT NULL DEFAULT 0,
			`publicTimestamp` timestamp NULL DEFAULT NULL,
			`spoiler` tinyint (1) NOT NULL DEFAULT 0,
			`caption` text DEFAULT NULL,
			PRIMARY KEY (`id`),
			KEY `uuid` (`uuid`),
			CONSTRAINT `playerScreenshots_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerSessions`
--
DROP TABLE IF EXISTS `playerSessions`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerSessions` (
		`sessionId` varchar(32) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`expiration` datetime NOT NULL,
			PRIMARY KEY (`sessionId`),
			UNIQUE KEY `sessionId` (`sessionId`),
			KEY `uuid` (`uuid`),
			CONSTRAINT `playerSessions_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerTags`
--
DROP TABLE IF EXISTS `playerTags`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerTags` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`name` varchar(32) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`timestampUnlocked` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
			PRIMARY KEY (`uuid`, `name`),
			CONSTRAINT `playerTags_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `playerTimeTrials`
--
DROP TABLE IF EXISTS `playerTimeTrials`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`playerTimeTrials` (
		`id` int (11) NOT NULL AUTO_INCREMENT,
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`mapId` int (11) NOT NULL,
			`seconds` int (11) NOT NULL,
			`timestampCompleted` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
			PRIMARY KEY (`uuid`, `mapId`),
			UNIQUE KEY `id` (`id`),
			CONSTRAINT `playerTimeTrials_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`)
	) ENGINE = InnoDB AUTO_INCREMENT = 7975 DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `players`
--
DROP TABLE IF EXISTS `players`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;

/*!40101 SET character_set_client = utf8 */;

CREATE TABLE
	`players` (
		`uuid` char(16) CHARACTER
		SET
			ascii COLLATE ascii_general_ci NOT NULL,
			`ip` varchar(39) CHARACTER
		SET
			ascii COLLATE ascii_general_ci DEFAULT NULL,
			`rank` int (11) NOT NULL DEFAULT 0,
			`banned` tinyint (1) NOT NULL,
			`muted` tinyint (1) NOT NULL DEFAULT 0,
			PRIMARY KEY (`uuid`),
			UNIQUE KEY `ip` (`ip`)
	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;

/*!40101 SET character_set_client = @saved_cs_client */;

/*!50003 SET @saved_cs_client      = @@character_set_client */;

/*!50003 SET @saved_cs_results     = @@character_set_results */;

/*!50003 SET @saved_col_connection = @@collation_connection */;

/*!50003 SET character_set_client  = utf8mb3 */;

/*!50003 SET character_set_results = utf8mb3 */;

/*!50003 SET collation_connection  = utf8mb3_general_ci */;

/*!50003 SET @saved_sql_mode       = @@sql_mode */;

/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION' */;

;

/*!50003 SET sql_mode              = @saved_sql_mode */;

/*!50003 SET character_set_client  = @saved_cs_client */;

/*!50003 SET character_set_results = @saved_cs_results */;

/*!50003 SET collation_connection  = @saved_col_connection */;

/*!50003 SET @saved_cs_client      = @@character_set_client */;

/*!50003 SET @saved_cs_results     = @@character_set_results */;

/*!50003 SET @saved_col_connection = @@collation_connection */;

/*!50003 SET character_set_client  = utf8mb3 */;

/*!50003 SET character_set_results = utf8mb3 */;

/*!50003 SET collation_connection  = utf8mb3_general_ci */;

/*!50003 SET @saved_sql_mode       = @@sql_mode */;

/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION' */;

;

/*!50003 SET sql_mode              = @saved_sql_mode */;

/*!50003 SET character_set_client  = @saved_cs_client */;

/*!50003 SET character_set_results = @saved_cs_results */;

/*!50003 SET collation_connection  = @saved_col_connection */;

-- --
-- -- Table structure for table `rankingCategories`
-- --
-- DROP TABLE IF EXISTS `rankingCategories`;
-- /*!40101 SET @saved_cs_client     = @@character_set_client */;
-- /*!40101 SET character_set_client = utf8 */;
-- CREATE TABLE
-- 	`rankingCategories` (
-- 		`categoryId` varchar(40) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`game` varchar(16) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`ordinal` int (11) NOT NULL,
-- 			`periodic` tinyint (1) DEFAULT 0,
-- 			-- PRIMARY KEY (`categoryId`, `game`)
-- 			PRIMARY KEY `categoryId`
-- 	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;
-- /*!40101 SET character_set_client = @saved_cs_client */;
-- --
-- -- Table structure for table `rankingEntries`
-- --
-- DROP TABLE IF EXISTS `rankingEntries`;
-- /*!40101 SET @saved_cs_client     = @@character_set_client */;
-- /*!40101 SET character_set_client = utf8 */;
-- CREATE TABLE
-- 	`rankingEntries` (
-- 		`categoryId` varchar(40) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`subCategoryId` varchar(32) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`position` int (11) NOT NULL,
-- 			`actualPosition` int (11) NOT NULL,
-- 			`uuid` char(16) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`valueInt` int (11) DEFAULT NULL,
-- 			`valueFloat` float DEFAULT NULL,
-- 			`timestamp` timestamp NULL DEFAULT NULL,
-- 			PRIMARY KEY (`categoryId`, `subCategoryId`, `uuid`),
-- 			KEY `uuid` (`uuid`),
-- 			KEY `subCategoryId` (`subCategoryId`),
-- 			KEY `catIdSubCatId` (`categoryId`, `subCategoryId`),
-- 			CONSTRAINT `rankingEntries_ibfk_1` FOREIGN KEY (`uuid`) REFERENCES `players` (`uuid`),
-- 			CONSTRAINT `rankingEntries_ibfk_2` FOREIGN KEY (`categoryId`) REFERENCES `rankingCategories` (`categoryId`),
-- 			CONSTRAINT `rankingEntries_ibfk_3` FOREIGN KEY (`subCategoryId`) REFERENCES `rankingSubCategories` (`subCategoryId`)
-- 	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;
-- /*!40101 SET character_set_client = @saved_cs_client */;
-- --
-- -- Table structure for table `rankingSubCategories`
-- --
-- DROP TABLE IF EXISTS `rankingSubCategories`;
-- /*!40101 SET @saved_cs_client     = @@character_set_client */;
-- /*!40101 SET character_set_client = utf8 */;
-- CREATE TABLE
-- 	`rankingSubCategories` (
-- 		`subCategoryId` varchar(16) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`categoryId` varchar(40) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`game` varchar(16) CHARACTER
-- 		SET
-- 			ascii COLLATE ascii_general_ci NOT NULL,
-- 			`ordinal` int (11) NOT NULL,
-- 			PRIMARY KEY (`subCategoryId`, `categoryId`, `game`),
-- 			KEY `categoryId` (`categoryId`),
-- 			CONSTRAINT `rankingSubCategories_ibfk_1` FOREIGN KEY (`categoryId`) REFERENCES `rankingCategories` (`categoryId`)
-- 	) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_general_ci;
-- /*!40101 SET character_set_client = @saved_cs_client */;
-- /*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;
-- /*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
-- /*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
-- /*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
-- /*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
-- /*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
-- /*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
-- /*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
-- -- Dump completed on 2024-03-18  4:33:58