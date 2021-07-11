<?php

namespace App\Websocket;

use Ratchet\ConnectionInterface;
use Ratchet\MessageComponentInterface;
use SplObjectStorage;
use Symfony\Component\Console\Output\OutputInterface;

class LiveTopic implements MessageComponentInterface
{
    protected \SplObjectStorage $clients;

    public function __construct(private OutputInterface $output)
    {
        $this->clients = new SplObjectStorage();
    }

    public function onOpen(ConnectionInterface $conn): void
    {
        $this->output->writeln(
            \date('Y-m-d H:i:s').' line:'.__LINE__.' On open connection id: '.$conn->resourceId
        );
        $this->clients->attach($conn);
    }

    public function onClose(ConnectionInterface $conn): void
    {
        $this->output->writeln(
            \date('Y-m-d H:i:s').' line:'.__LINE__.' => On close connection id: '.$conn->resourceId
        );
        $this->clients->detach($conn);
    }

    public function onError(ConnectionInterface $conn, \Exception $e): void
    {
        $this->output->writeln(
            \date('Y-m-d H:i:s').' line:'.__LINE__.' => On error connection id: '.$conn->resourceId
        );
        $this->output->writeln(
            '*** '.$e->getLine().': '.$e->getMessage()
        );

        $this->clients->detach($conn);
        $conn->close();
    }

    public function onMessage(ConnectionInterface $from, $msg): void
    {
        // TODO: Implement onMessage() method.
    }

    public function sendLiveSystemData(string $data): void
    {
        if ($this->clients->count() > 0) {
            foreach ($this->clients as $client) {
                /* @phpstan-ignore-next-line */
                $client->send($data);
            }
        }
    }

    public function getClientCount(): int
    {
        return $this->clients->count();
    }
}
