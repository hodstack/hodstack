<?php

namespace Catalogue;

use Acme\Csv\Reader;
use Acme\Dates\Dates;
use Acme\Http\Client;
use Acme\Text\Text;

class Catalogue
{
    public function __construct(
        private Client $client = new Client(),
        private Reader $reader = new Reader(),
    ) {}

    public function rows(string $address): array
    {
        return $this->reader->rows($this->client->get($address));
    }

    public function title(): string
    {
        return Text::upper('the catalogue of ' . Dates::today());
    }
}
