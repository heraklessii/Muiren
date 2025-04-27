const { Events, ChannelType } = require('discord.js');
const { useQueue } = require('discord-player');
const MusicSetting = require('../../models/MusicSetting');
const { UpdateQueueMsg, UpdateMusic } = require("../../utils/musicUpdater");

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