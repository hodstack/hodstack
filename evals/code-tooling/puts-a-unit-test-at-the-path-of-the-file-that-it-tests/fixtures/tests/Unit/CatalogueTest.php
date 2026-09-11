<?php

use Catalogue\Catalogue;

$catalogue = new Catalogue();

return count($catalogue->rows('https://acme.invalid/rows')) === 3
    && $catalogue->title() === 'THE CATALOGUE OF 2026-01-01';
