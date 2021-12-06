<?php

namespace App\Ampere\DockerClient;

use App\Ampere\DockerClient\Request\Context;
use App\Ampere\DockerClient\Response\Response;
use ErrorException;
use function fclose;
use function stream_get_contents;
use function stream_set_blocking;
use function stream_socket_client;
use function stream_socket_sendto;

class SocketConnection
{
    private string $socketAddr = 'unix:///var/run/docker.soack';
    private int $connectTimeout = 30;

    /* @phpstan-ignore-next-line */
    private $conn;
    private int $errorCode = 0;
    private string $errorMessage = '';

    private function __construct(private string $context, bool $isBlocking)
    {
        try {
            $this->conn = stream_socket_client(
                $this->socketAddr,
                $this->errorCode,
                $this->errorMessage,
                $this->connectTimeout,
                STREAM_CLIENT_CONNECT
            );
            /* @phpstan-ignore-next-line */
            stream_set_blocking($this->conn, $isBlocking);
        } catch (ErrorException $exception) {
            throw new \Exception("Failed to create a socket connection.", 0, $exception);
        }
    }

    public static function create(Context $context): SocketConnection
    {
        return new SocketConnection(
            $context->getContext(),
            $context->isStreamBlocked()
        );
    }

    public function request(): Response
    {
        stream_socket_sendto($this->conn, $this->context);

        /* @phpstan-ignore-next-line */
        return new Response(stream_get_contents($this->conn, -1));
    }

    public function startStream(): void
    {
        stream_socket_sendto($this->conn, $this->context);
    }

    /* @phpstan-ignore-next-line */
    public function getConn()
    {
        return $this->conn;
    }

    public function closeConn(): void
    {
        fclose($this->conn);
    }
}
