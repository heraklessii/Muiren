// Muiren Bot - Discord Bot Project
// Copyright (C) 2025  Muiren Bot Contributors
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

const { Events, ChannelType } = require('discord.js');
const { useQueue } = require('discord-player');
const MusicSetting = require('../../models/MusicSetting');

module.exports = {
    name: Events.ChannelDelete,
    async execute(channel) {

        if (channel.type === ChannelType.GuildVoice) {

            const botMember = channel.guild.members.me;
            if (botMember.voice.channelId === channel.id) {
                const queue = useQueue(channel.guild.id);
                if (queue) {
                    queue.delete();
                    queue.connection?.disconnect();
                }
            }
        }

        if (channel.type === ChannelType.GuildText) {
            const setting = await MusicSetting.findOne({ guildId: channel.guild.id });
            if (setting && setting.channelId === channel.id) {
                setting.systemEnabled = false;
                setting.channelId = null;
                setting.messageId = null;
                await setting.save();
            }
        }

    }
};