const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/guests?query=';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = ``;


describe('OrgMember - Get Guest List', function () {
    before(async function () {
        response = await get(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })
    ;

    it("guests should be present", function () {

        let json = JSON.parse(responseBody)
        expect(json.data.length).to.be.greaterThan(10);
        pm.environment.set("last_ticket_instance_id", json.data[0].id);
        pm.environment.set("last_ticket_instance_redeem_key", json.data[0].redeem_key);
    });


});

            
