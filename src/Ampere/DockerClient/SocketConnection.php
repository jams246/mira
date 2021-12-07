<?php

namespace App\Ampere\DockerClient;

use App\Ampere\DockerClient\Request\Context;
use App\Ampere\DockerClient\Response\Response;
use ErrorException;

class SocketConnection
{
    public const DOCKER_SOCK_PATH = '/var/run/docker.sock';
    private const CONNECT_TIMEOUT = 10;

    /* @phpstan-ignore-next-line */
    private $conn;
    private int $errorCode = 0;
    private string $errorMessage = '';

    private function __construct(private string $context, bool $isBlocking)
    {
        try {
            $this->conn = \stream_socket_client(
                $this->buildAddressPath(),
                $this->errorCode,
                $this->errorMessage,
                self::CONNECT_TIMEOUT,
                STREAM_CLIENT_CONNECT
            );
            /* @phpstan-ignore-next-line */
            \stream_set_blocking($this->conn, $isBlocking);
        } catch (ErrorException $exception) {
            $errorMessage = \sprintf(
                'stream_socket_client initialization failed. Code: %d, Message: %s',
                $this->errorCode,
                $this->errorMessage
            );
            throw new \Exception($errorMessage, $this->errorCode);
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
        \stream_socket_sendto($this->conn, $this->context);

        /* @phpstan-ignore-next-line */
        $response = new Response(\stream_get_contents($this->conn, -1));

        $this->closeConn();

        return $response;
    }

    public function startStream(): void
    {
        \stream_socket_sendto($this->conn, $this->context);
    }

    /* @phpstan-ignore-next-line */
    public function getConn()
    {
        return $this->conn;
    }

    public function closeConn(): void
    {
        \fclose($this->conn);
    }

    private function buildAddressPath(): string
    {
        return 'unix://'.self::DOCKER_SOCK_PATH;
    }
}
