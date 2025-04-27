const { Events } = require('discord.js');
const { useMainPlayer, QueryType, useQueue } = require('discord-player');
const MusicSetting = require('../../models/MusicSetting');
const { UpdateQueueMsg } = require("../../utils/musicUpdater");

module.exports = {
    name: Events.MessageCreate,
    async execute(message) {

        if (!message.guild || message.author.bot) return;

        const setting = await MusicSetting.findOne({ guildId: message.guildId });
        if (!setting || !setting.systemEnabled) return;

        if (message.channel.id !== setting.channelId) return;

        const memberVoice = message.member.voice.channel;
        if (!memberVoice) {
            await message.delete().catch(() => { });
            return message.reply({ content: ':x: | Lütfen önce bir ses kanalına katılın.' });
        }

        const player = useMainPlayer();
        const query = message.content.trim();
        if (!query) return;

        await message.delete().catch(() => { });
        await message.channel.sendTyping();

        const searchResult = await player.search(query, {
            requestedBy: message.member,
            searchEngine: QueryType.AUTO
        });

        if (!searchResult || !searchResult.tracks.length)
            return message.channel.send(':x: | Şarkı bulunamadı.');

        let queue = useQueue(message.guildId);
        if (!queue) {
            queue = player.nodes.create(message.guild, {
                metadata: { channel: message.channel, requestedBy: message.member }
            });
        }

        // Connect to voice
        try {
            if (!queue.connection) {
                await queue.connect(memberVoice);
            }
        } catch (err) {
            console.error('Ses kanalına bağlanırken hata:', err);
            return message.channel.send(':x: | Ses kanalına bağlanılamadı.');
        }

        // Add and play track
        const track = searchResult.tracks[0];
        queue.addTrack(track);
        if (!queue.node.isPlaying()) {
            await queue.node.play();
        }

        await UpdateQueueMsg(queue);
    }
};
